import {
  assertChallengeFresh,
  createChallenge,
  createRegistryEmailAuthRequest,
  createRegistryEmailSession,
  lookupPgpKeyOnKeyservers,
  normalizePgpFingerprint,
  verifyRegistryPgpChallenge,
  verifyRegistrySshChallenge,
} from "./auth";
import {
  claimNodeOperationLock,
  consumeFreshChallenge,
  consumeCompletedRegistryEmailAuthRequestByToken,
  deleteChallenge,
  deleteOperation,
  deleteRegistryEmailAuthRequest,
  getAuthSession,
  getChallenge,
  getOidcAuthRequest,
  getRegistryEmailAuthRequest,
  getRegistryEmailAuthRequestByToken,
  getOperation,
  insertOperation,
  listActiveOperations,
  listOperationsForAsn,
  deleteOidcAuthRequest,
  putAuthSession,
  putChallenge,
  putOidcAuthRequest,
  putRegistryEmailAuthRequest,
  putOperation,
  releaseNodeOperationLock,
} from "./db";
import { branchName, GitHubClient } from "./github";
import type { GitHubWorkflowJob, GitHubWorkflowRun } from "./github";
import { resolveLocale, resolveLocaleCode, t } from "./i18n";
import { sendRegistryEmailAuthMessage } from "./mailer";
import {
  buildNodeViews,
  listSessionsForAsn,
  loadInventoryHosts,
  mutatePeerFile,
  validateSessionSpec,
} from "./network";
import {
  createOidcAuthorizationRequest,
  exchangeAuthorizationCode,
  fetchOidcDiscovery,
  oidcAsnFromClaimSources,
  oidcMaintainerFromClaimSources,
  oidcMethodsFromProviders,
  oidcProviderByName,
  rewriteIssuerHost,
  sessionFromOidcIdentity,
  verifiedOidcClaimSources,
} from "./oidc";
import { NoMaintainerError, RegistryPathNotFoundError, loadMaintainersForAsn, methodsFromMaintainers } from "./registry";
import { MP_BGP_TRANSPORTS } from "./types";
import type {
  AuthSessionResponse,
  AuthStartRequest,
  AuthStartResponse,
  ChallengeRecord,
  CreateSessionRequest,
  HostImpersonationRequest,
  MaintainerRecord,
  OperationFailureDetails,
  RegistryEmailCompleteRequest,
  RegistryEmailSendRequest,
  RegistryEmailSendResponse,
  RegistryEmailTarget,
  RegistryEmailVerifyRequest,
  OidcCompleteRequest,
  OidcProviderConfig,
  OidcStartRequest,
  OidcStartResponse,
  OperationKind,
  OperationRecord,
  OperationState,
  OperationStatus,
  PeerSessionSpec,
  PgpKeyLookupResponse,
  RegistryPgpVerifyRequest,
  RegistrySshVerifyRequest,
  SessionRecord,
  UiMessage,
  UpdateSessionRequest,
} from "./types";
import {
  bearerToken,
  buildCorsHeaders,
  errorWithCors,
  HttpError,
  isUiMessageKey,
  isTerminalOperationState,
  jsonWithCors,
  normalizeSupportedAutopeerAsn,
  nowIso,
  parseConfiguredAsns,
  parseJsonEnv,
  readJson,
  readOptionalEnvString,
  requireBoolean,
  requireNonEmptyString,
  requireOptionalInteger,
  requireOptionalString,
  requireRecord,
  I18nError,
  isExpired,
  stripOperatorHints,
  readOptionalSecret,
  timingSafeEqual,
  toUiMessage,
  uiMessage,
} from "./utils";
import { OPENAPI_PATH, SWAGGER_PATH, openApiSpec, swaggerUiHtml } from "./docs";

const INVENTORY_PATH = "inventory.yaml";
const AUTOPEER_POLICY_PATH = "group_vars/all/autopeer.yaml";
const PEER_FILE_PATH = (node: string): string => `host_vars/${node}/dn42_peers.yaml`;
const CHECK_WORKFLOW_ID = "peer-session-check.yml";
const CHECK_WORKFLOW_GRACE_MS = 5 * 60 * 1000;
const APPLY_WORKFLOW_ID = "peer-session-apply.yml";
const APPLY_WORKFLOW_GRACE_MS = 10 * 60 * 1000;
const CONFIG_PATH = "/config.json";

function commitMessage(kind: string, asn: string, node: string): string {
  return `feat: autopeer ${kind} AS${asn} on ${node}`;
}

async function requireOwnedOperation(
  env: Env,
  request: Request,
  operationId: string,
): Promise<{ authSession: SessionRecord; operation: OperationRecord }> {
  const authSession = await requireSession(env, request);
  const operation = await getOperation(env, operationId);
  if (!operation || operation.asn !== authSession.asn) {
    throw new HttpError("error.request.operation.not_found", 404);
  }
  return { authSession, operation };
}
const OIDC_CALLBACK_PREFIX = "/oidc/callback/";
const REGISTRY_EMAIL_CALLBACK_PATH = "/auth/email/callback";

type ValidationWorkflowRun = {
  status: string;
  conclusion: string | null;
};

type PreMergeGateDecision = {
  state: OperationState;
  message: UiMessage;
  shouldAttemptMerge: boolean;
};

type RefreshOperationOptions = {
  allowMergeAttempt?: boolean;
};

function errorMessage(error: unknown, fallbackKey: string): UiMessage {
  if (error instanceof HttpError) return error.uiMessage;
  if (error instanceof I18nError) return error.uiMessage;
  return uiMessage(fallbackKey);
}

function fragmentMessage(name: string, message: string | UiMessage): string {
  return `${name}=${encodeURIComponent(JSON.stringify(toUiMessage(message)))}`;
}

function withLangFragment(fragment: string, locale?: string | null): string {
  return locale ? `${fragment}&lang=${encodeURIComponent(locale)}` : fragment;
}


export function classifyMaintainerLookupError(asn: string, error: unknown): HttpError {
  if (error instanceof RegistryPathNotFoundError) {
    return new HttpError(uiMessage("error.auth.asn.not_found", { asn }), 400);
  }

  if (error instanceof NoMaintainerError) {
    return new HttpError(uiMessage("error.auth.asn.no_supported_auth", { asn }), 400);
  }

  if (error instanceof HttpError) {
    return error;
  }
  return new HttpError(uiMessage("error.registry.lookup_failed", { asn }), 502);
}

async function loadMaintainersForRequestAsn(
  env: Env,
  asn: string,
): Promise<MaintainerRecord[]> {
  try {
    return await loadMaintainersForAsn(env, asn);
  } catch (error) {
    throw classifyMaintainerLookupError(asn, error);
  }
}

function parseRequestSessionSpec(value: unknown): PeerSessionSpec {
  const record = requireRecord(value, "session");
  const peeringStrategy = requireOptionalString(
    record.peering_strategy,
    "session.peering_strategy",
  ) ?? "full_table";
  const mpBgpTransport = requireOptionalString(
    record.mp_bgp_transport,
    "session.mp_bgp_transport",
  );
  if (
    mpBgpTransport !== null &&
    mpBgpTransport !== undefined &&
    !(MP_BGP_TRANSPORTS as readonly string[]).includes(mpBgpTransport)
  ) {
    throw new HttpError("error.request.session.mp_bgp_transport.invalid", 400);
  }
  return {
    comment: requireOptionalString(record.comment, "session.comment"),
    endpoint: requireOptionalString(record.endpoint, "session.endpoint"),
    wg_public_key: requireNonEmptyString(record.wg_public_key, "session.wg_public_key"),
    port: requireOptionalInteger(record.port, "session.port"),
    peer4: requireOptionalString(record.peer4, "session.peer4"),
    peer6: requireOptionalString(record.peer6, "session.peer6"),
    own6: requireOptionalString(record.own6, "session.own6"),
    keepalive: requireOptionalInteger(record.keepalive, "session.keepalive"),
    mtu: requireOptionalInteger(record.mtu, "session.mtu"),
    ipv4: requireBoolean(record.ipv4, "session.ipv4"),
    ipv6: requireBoolean(record.ipv6, "session.ipv6"),
    extended_next_hop: requireBoolean(
      record.extended_next_hop,
      "session.extended_next_hop",
    ),
    mp_bgp: requireBoolean(record.mp_bgp, "session.mp_bgp"),
    mp_bgp_transport: mpBgpTransport as PeerSessionSpec["mp_bgp_transport"],
    peering_strategy: peeringStrategy as PeerSessionSpec["peering_strategy"],
    psk: record.psk === undefined ? undefined : requireOptionalString(record.psk, "session.psk"),
    encrypt_endpoint: typeof record.encrypt_endpoint === "boolean" ? record.encrypt_endpoint : undefined,
  };
}

function assertValidSessionSpec(
  node: Parameters<typeof validateSessionSpec>[0],
  asn: string,
  spec: PeerSessionSpec,
): void {
  try {
    validateSessionSpec(node, asn, spec);
  } catch (error) {
    throw new HttpError(errorMessage(error, "error.request.session_payload.invalid"), 400);
  }
}

function configuredUrl(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function forwardedHeaderValue(request: Request, name: string): string | undefined {
  const value = request.headers
    .get(name)
    ?.split(",")
    .map((entry) => entry.trim())
    .find(Boolean);
  return value || undefined;
}

function trustedForwardedHosts(env: Env): Set<string> {
  const hosts = new Set<string>();
  const configured = configuredUrl(env.AUTOPEER_SITE_URL);
  if (configured) {
    try {
      hosts.add(new URL(configured).host.toLowerCase());
    } catch {
      // ignore malformed configuration
    }
  }
  const list = readOptionalEnvString(env, "AUTOPEER_TRUSTED_FORWARDED_HOSTS");
  if (list) {
    for (const entry of list.split(",")) {
      const trimmed = entry.trim().toLowerCase();
      if (trimmed) hosts.add(trimmed);
    }
  }
  return hosts;
}

function trustedForwardedHost(env: Env, request: Request): string | undefined {
  const forwarded = forwardedHeaderValue(request, "x-forwarded-host")?.toLowerCase();
  if (!forwarded) return undefined;
  return trustedForwardedHosts(env).has(forwarded) ? forwarded : undefined;
}

function externalSiteBaseUrl(env: Env, request: Request): URL {
  const base = new URL(configuredUrl(env.AUTOPEER_SITE_URL) ?? request.url);
  const forwardedHost = trustedForwardedHost(env, request);
  if (!forwardedHost) {
    return base;
  }

  const forwardedProto = forwardedHeaderValue(request, "x-forwarded-proto");
  const external = new URL(base.toString());
  external.host = forwardedHost;
  if (forwardedProto === "http" || forwardedProto === "https") {
    external.protocol = `${forwardedProto}:`;
  }
  return external;
}

function isDn42Request(env: Env, request: Request): boolean {
  const host = trustedForwardedHost(env, request) ?? new URL(request.url).host.toLowerCase();
  return host.endsWith(".dn42");
}

function runtimeConfigResponse(request: Request, env: Env) {
  const origin = new URL(request.url).origin;
  return {
    autopeer_api_url: configuredUrl(env.AUTOPEER_API_URL) ?? origin,
    autopeer_site_url: configuredUrl(env.AUTOPEER_SITE_URL) ?? origin,
    looking_glass_url: configuredUrl(env.LOOKING_GLASS_URL),
    oidc_methods: oidcMethodsFromProviders(configuredOidcProviders(env)),
  };
}

function configuredOidcProviders(env: Env): OidcProviderConfig[] {
  return parseJsonEnv(env.OIDC_PROVIDERS, "OIDC_PROVIDERS");
}

function registryEmailAuthConfigured(env: Env): boolean {
  return readOptionalEnvString(env, "RESEND_API_KEY") !== null;
}

async function consumeChallengeOrThrow(env: Env, challengeId: string): Promise<ChallengeRecord> {
  const result = await consumeFreshChallenge(env, challengeId);
  switch (result.kind) {
    case "available":
      return result.challenge;
    case "missing":
      throw new HttpError("error.auth.challenge.unknown_id", 404);
    case "expired":
      throw new HttpError("error.auth.challenge.expired", 400);
    case "consumed":
      throw new HttpError("error.auth.challenge.used", 400);
  }
}

function oidcCallbackUrl(env: Env, request: Request, providerName: string): string {
  const base = externalSiteBaseUrl(env, request);
  return new URL(
    `${OIDC_CALLBACK_PREFIX}${encodeURIComponent(providerName)}`,
    base.pathname.endsWith("/") ? base.toString() : `${base.toString()}/`,
  ).toString();
}

function siteRedirectResponse(env: Env, request: Request, fragment: string): Response {
  const target = externalSiteBaseUrl(env, request);
  target.hash = fragment;
  return Response.redirect(target.toString(), 302);
}

function registryEmailTargetsForChallenge(challenge: ChallengeRecord): RegistryEmailTarget[] {
  const methodTargets = challenge.methods.find((method) => method.kind === "registry_email")
    ?.email_targets ?? [];
  if (methodTargets.length > 0) {
    return methodTargets.filter((target) => target.emails.length > 0);
  }

  return challenge.maintainers
    .map((maintainer) => ({
      maintainer: maintainer.name,
      emails: maintainer.contact_emails ?? [],
    }))
    .filter((target) => target.emails.length > 0);
}

function resolveRegistryEmailTarget(
  challenge: ChallengeRecord,
  requestedMaintainer?: string | null,
): RegistryEmailTarget {
  const targets = registryEmailTargetsForChallenge(challenge);
  if (targets.length === 0) {
    throw new HttpError(
      uiMessage("error.auth.registry_email.contacts.missing", { asn: challenge.asn }),
      400,
    );
  }

  if (requestedMaintainer) {
    const requested = requestedMaintainer.trim().toUpperCase();
    const matched = targets.find((target) => target.maintainer.toUpperCase() === requested);
    if (!matched) {
      throw new HttpError(
        uiMessage("error.auth.registry_email.target.missing", { requested }),
        400,
      );
    }
    return matched;
  }

  if (targets.length === 1) {
    return targets[0];
  }

  throw new HttpError(
    uiMessage("error.auth.registry_email.target.required"),
    400,
  );
}

function registryEmailCallbackUrl(
  env: Env,
  request: Request,
  challengeId: string,
  token: string,
): string {
  const base = externalSiteBaseUrl(env, request);
  const callback = new URL(
    REGISTRY_EMAIL_CALLBACK_PATH,
    base.pathname.endsWith("/") ? base.toString() : `${base.toString()}/`,
  );
  callback.searchParams.set("challenge_id", challengeId);
  callback.searchParams.set("token", token);
  return callback.toString();
}

async function createCompletedRegistryEmailSession(
  env: Env,
  challengeId: string,
  effectiveMnt: string,
): Promise<SessionRecord> {
  const challenge = await consumeChallengeOrThrow(env, challengeId);
  const session = createRegistryEmailSession(challenge, effectiveMnt);
  await putAuthSession(env, session);
  return session;
}

function sessionCanImpersonate(env: Env, session: SessionRecord): boolean {
  return parseConfiguredAsns(env.HOST_ASNS).has(session.asn);
}

function sessionCanMutate(env: Env, session: SessionRecord): boolean {
  return !sessionCanImpersonate(env, session) || session.auth_method.kind === "host_impersonation";
}

function authSessionResponseForEnv(env: Env, session: SessionRecord): AuthSessionResponse {
  return {
    session_token: session.token,
    asn: session.asn,
    effective_mnt: session.effective_mnt,
    auth_method: session.auth_method,
    can_impersonate: sessionCanImpersonate(env, session),
    expires_at: session.expires_at,
  };
}

function availableMaintainerNames(maintainers: MaintainerRecord[]): string[] {
  return [...new Set(maintainers.map((maintainer) => maintainer.name))];
}

export function resolveEffectiveMaintainer(
  maintainers: MaintainerRecord[],
  requestedMaintainer?: string | null,
): string {
  if (maintainers.length === 0) {
    throw new HttpError("error.auth.impersonation.no_maintainers", 400);
  }

  const available = availableMaintainerNames(maintainers).join(", ");

  if (requestedMaintainer) {
    const requested = requestedMaintainer.trim().toUpperCase();
    const matched = maintainers.find((maintainer) => maintainer.name.toUpperCase() === requested);
    if (!matched) {
      throw new HttpError(
        uiMessage("error.auth.impersonation.maintainer.missing", {
          requested,
          available,
        }),
        400,
      );
    }
    return matched.name;
  }

  if (maintainers.length === 1) {
    return maintainers[0].name;
  }

  throw new HttpError(
    uiMessage("error.auth.impersonation.maintainer.required", { available }),
    400,
  );
}

async function requireSession(env: Env, request: Request): Promise<SessionRecord> {
  const token = bearerToken(request);
  if (!token) {
    throw new HttpError("error.auth.session.token.missing", 401);
  }

  const session = await getAuthSession(env, token);
  if (!session) {
    throw new HttpError("error.auth.session.unknown", 401);
  }
  if (isExpired(session.expires_at)) {
    throw new HttpError("error.auth.session.expired", 401);
  }

  return session;
}

async function loadRepoState(env: Env, github: GitHubClient) {
  const [inventoryFile, policyFile] = await Promise.all([
    github.getFile(INVENTORY_PATH, env.GITHUB_BASE_BRANCH),
    github.getFile(AUTOPEER_POLICY_PATH, env.GITHUB_BASE_BRANCH),
  ]);
  if (!inventoryFile.exists || !inventoryFile.text) {
    throw new HttpError("error.repo.inventory.missing", 502);
  }

  const hosts = loadInventoryHosts(inventoryFile.text, policyFile.text ?? null);
  const peerFileResults = await Promise.all(
    hosts.map(async (host) => {
      const file = await github.getFile(PEER_FILE_PATH(host.name), env.GITHUB_BASE_BRANCH);
      if (!file.exists || !file.text) {
        return null;
      }
      return [host.name, file.text] as const;
    }),
  );
  const peerFiles = new Map(peerFileResults.filter((entry): entry is NonNullable<typeof entry> => entry !== null));

  return {
    hosts,
    peerFiles,
  };
}

function buildOperationMessage(state: OperationState): UiMessage {
  switch (state) {
    case "pending_pull_request":
      return uiMessage("operation.message.pending_pull_request");
    case "pending_checks":
      return uiMessage("operation.message.pending_checks");
    case "applying":
      return uiMessage("operation.message.applying");
    case "pending_merge":
      return uiMessage("operation.message.pending_merge");
    case "completed":
      return uiMessage("operation.message.completed");
    case "failed":
      return uiMessage("operation.message.failed");
    case "conflict":
      return uiMessage("operation.message.conflict");
  }
}

function pickFailingJob(jobs: GitHubWorkflowJob[]): GitHubWorkflowJob | undefined {
  return jobs.find(
    (job) =>
      job.status === "completed" &&
      job.conclusion !== null &&
      job.conclusion !== "success" &&
      job.conclusion !== "skipped" &&
      job.conclusion !== "neutral",
  );
}

function stageFromJobName(name: string): OperationFailureDetails["stage"] {
  const normalized = name.toLowerCase();
  if (normalized.includes("preflight")) return "preflight";
  if (normalized.includes("check")) return "checks";
  return "apply";
}

function pickFailingStepName(job: GitHubWorkflowJob): string | null {
  const failing = job.steps?.find(
    (step) =>
      step.status === "completed" &&
      step.conclusion !== null &&
      step.conclusion !== "success" &&
      step.conclusion !== "skipped" &&
      step.conclusion !== "neutral",
  );
  return failing?.name ?? null;
}

const GENERIC_EXIT_CODE_RE = /^(Process completed with exit code \d+\.?)$/;

function extractFailingStepLog(rawLog: string, stepName: string, maxLines = 25): string | null {
  const lines = rawLog.split("\n");
  const strip = (line: string) => line.replace(/^\d{4}-\d{2}-\d{2}T[\d:.]+Z\s?/, "");

  let inTarget = false;
  let pastGroup = false;
  const output: string[] = [];

  for (const line of lines) {
    const s = strip(line);

    if (s.startsWith("##[group]") && s.includes(stepName)) {
      inTarget = true;
      pastGroup = false;
      output.length = 0;
      continue;
    }
    if (inTarget && !pastGroup && s.startsWith("##[endgroup]")) {
      pastGroup = true;
      continue;
    }
    if (inTarget && pastGroup) {
      if (s.startsWith("##[group]")) break;
      if (s.startsWith("##[error]Process completed with exit code")) continue;
      output.push(s);
    }
  }

  while (output.length > 0 && output[output.length - 1].trim() === "") output.pop();
  if (output.length === 0) return null;
  const tail = output.slice(-maxLines);
  if (tail.length < output.length) tail.unshift(`… (${output.length - tail.length} lines omitted)`);
  return tail.join("\n");
}

async function buildWorkflowFailureDetails(
  github: GitHubClient,
  run: Pick<GitHubWorkflowRun, "id" | "html_url" | "conclusion">,
  fallbackStage: OperationFailureDetails["stage"],
): Promise<OperationFailureDetails> {
  const details: OperationFailureDetails = {
    stage: fallbackStage,
    conclusion: run.conclusion,
    run_url: run.html_url,
  };

  try {
    const { jobs } = await github.listWorkflowRunJobs(run.id);
    const failingJob = pickFailingJob(jobs);
    if (failingJob) {
      details.stage = stageFromJobName(failingJob.name);
      details.step = pickFailingStepName(failingJob) ?? failingJob.name;
      try {
        const annotations = await github.listCheckRunAnnotations(failingJob.id);
        const firstFailureAnnotation =
          annotations.find((a) => a.annotation_level === "failure") ?? annotations[0];
        if (firstFailureAnnotation) {
          const title = firstFailureAnnotation.title?.trim();
          const message = firstFailureAnnotation.message?.trim();
          details.annotation = [title, message].filter(Boolean).join(": ") || null;
        }
      } catch (error) {
        console.warn("failed to read check-run annotations", error);
      }

      if (!details.annotation || GENERIC_EXIT_CODE_RE.test(details.annotation)) {
        try {
          const rawLog = await github.downloadJobLog(failingJob.id);
          if (rawLog && details.step) {
            const extracted = extractFailingStepLog(rawLog, details.step);
            if (extracted) details.annotation = extracted;
          }
        } catch (error) {
          console.warn("failed to read job log", error);
        }
      }
    }
  } catch (error) {
    console.warn("failed to read workflow jobs", error);
  }

  return details;
}

function failureMessageFromDetails(details: OperationFailureDetails): UiMessage {
  const params: Record<string, string> = {
    stage: details.stage,
    conclusion: details.conclusion ?? "unknown",
  };
  if (details.step) params.step = details.step;
  if (details.annotation) params.annotation = details.annotation;

  const key = details.step && details.annotation
    ? "operation.message.workflow_failed.full"
    : details.step
      ? "operation.message.workflow_failed.step"
      : "operation.message.workflow_failed";

  return uiMessage(key, params);
}

function buildNoChangeOperation(
  authSession: SessionRecord,
  nodeName: string,
  kind: OperationRecord["kind"],
  sessionSnapshot: PeerSessionSpec | null,
): OperationRecord {
  const now = nowIso();
  return {
    id: crypto.randomUUID(),
    asn: authSession.asn,
    node: nodeName,
    kind,
    state: "completed",
    branch: "",
    pr_number: null,
    pr_node_id: null,
    pull_request_url: null,
    workflow_run_url: null,
    message: uiMessage("operation.message.no_change"),
    failure_details: null,
    created_at: now,
    updated_at: now,
    session_snapshot: sessionSnapshot,
  };
}

export function decideCheckGate(
  operation: Pick<OperationRecord, "created_at">,
  validationRun: ValidationWorkflowRun | undefined,
  now = Date.now(),
): PreMergeGateDecision {
  if (!validationRun) {
    const createdAt = Date.parse(operation.created_at);
    if (Number.isFinite(createdAt) && now - createdAt > CHECK_WORKFLOW_GRACE_MS) {
      return {
        state: "failed",
        message: uiMessage("operation.message.check_not_started"),
        shouldAttemptMerge: false,
      };
    }
    return {
      state: "pending_checks",
      message: uiMessage("operation.message.check_wait_start"),
      shouldAttemptMerge: false,
    };
  }

  if (validationRun.status !== "completed") {
    return {
      state: "pending_checks",
      message: buildOperationMessage("pending_checks"),
      shouldAttemptMerge: false,
    };
  }

  if (!["success", "neutral", "skipped"].includes(validationRun.conclusion ?? "")) {
    return {
      state: "failed",
      message: uiMessage("operation.message.check_failed", {
        conclusion: validationRun.conclusion ?? "unknown",
      }),
      shouldAttemptMerge: false,
    };
  }

  return {
    state: "applying",
    message: buildOperationMessage("applying"),
    shouldAttemptMerge: false,
  };
}

export function decideApplyGate(
  operation: Pick<OperationRecord, "created_at">,
  applyRun: ValidationWorkflowRun | undefined,
  now = Date.now(),
): PreMergeGateDecision {
  if (!applyRun) {
    const createdAt = Date.parse(operation.created_at);
    if (Number.isFinite(createdAt) && now - createdAt > CHECK_WORKFLOW_GRACE_MS + APPLY_WORKFLOW_GRACE_MS) {
      return {
        state: "failed",
        message: uiMessage("operation.message.apply_not_started"),
        shouldAttemptMerge: false,
      };
    }
    return {
      state: "applying",
      message: uiMessage("operation.message.apply_wait_start"),
      shouldAttemptMerge: false,
    };
  }

  if (applyRun.status !== "completed") {
    return {
      state: "applying",
      message: buildOperationMessage("applying"),
      shouldAttemptMerge: false,
    };
  }

  if (!["success", "neutral", "skipped"].includes(applyRun.conclusion ?? "")) {
    return {
      state: "failed",
      message: uiMessage("operation.message.apply_failed", {
        conclusion: applyRun.conclusion ?? "unknown",
      }),
      shouldAttemptMerge: false,
    };
  }

  return {
    state: "pending_merge",
    message: buildOperationMessage("pending_merge"),
    shouldAttemptMerge: true,
  };
}

export function decideNodeLockGate(hasNodeLock: boolean): PreMergeGateDecision {
  if (hasNodeLock) {
    return {
      state: "pending_merge",
      message: buildOperationMessage("pending_merge"),
      shouldAttemptMerge: true,
    };
  }

  return {
    state: "pending_merge",
    message: uiMessage("operation.message.wait_node_lock"),
    shouldAttemptMerge: false,
  };
}

async function claimNodeLockForMerge(
  env: Env,
  github: GitHubClient,
  operation: OperationRecord,
): Promise<boolean> {
  for (let attempt = 0; attempt < 3; attempt += 1) {
    const ownerOperationId = await claimNodeOperationLock(env, operation.node, operation.id);
    if (ownerOperationId === operation.id) {
      return true;
    }

    const ownerOperation = await getOperation(env, ownerOperationId);
    if (!ownerOperation) {
      await releaseNodeOperationLock(env, operation.node, ownerOperationId);
      continue;
    }

    if (
      (ownerOperation.state === "pending_merge" || ownerOperation.state === "applying") &&
      ownerOperation.pr_number
    ) {
      const refreshedOwner = await refreshOperation(env, github, ownerOperation, {
        allowMergeAttempt: false,
      });
      if (isTerminalOperationState(refreshedOwner.state)) {
        await releaseNodeOperationLock(env, operation.node, ownerOperationId);
        continue;
      }
    } else if (isTerminalOperationState(ownerOperation.state)) {
      await releaseNodeOperationLock(env, operation.node, ownerOperationId);
      continue;
    }

    return false;
  }

  return false;
}

async function refreshOperation(
  env: Env,
  github: GitHubClient,
  operation: OperationRecord,
  options: RefreshOperationOptions = {},
): Promise<OperationStatus> {
  const allowMergeAttempt = options.allowMergeAttempt ?? true;
  if (isTerminalOperationState(operation.state) || !operation.pr_number) {
    await releaseNodeOperationLock(env, operation.node, operation.id);
    return operation;
  }

  const pr = await github.getPullRequest(operation.pr_number);
  let nextState: OperationState = operation.state;
  let workflowRunUrl = operation.workflow_run_url ?? null;
  let message = operation.message ?? buildOperationMessage(operation.state);
  let failureDetails: OperationFailureDetails | null = operation.failure_details ?? null;

  if (pr.merged) {
    await github.deleteBranch(operation.branch).catch(() => {});
    nextState = "completed";
    message = buildOperationMessage(nextState);
    failureDetails = null;
  } else if (pr.state !== "open") {
    nextState = "failed";
    message = uiMessage("operation.message.pull_request_closed");
  } else {
    const validationRuns = await github.listWorkflowRuns(CHECK_WORKFLOW_ID, {
      branch: operation.branch,
      event: "pull_request",
      perPage: 5,
    });
    const validationRun = validationRuns.workflow_runs.find(
      (candidate) => candidate.head_sha === pr.head.sha,
    );
    const checkGate = decideCheckGate(operation, validationRun);
    nextState = checkGate.state;
    message = checkGate.message;

    if (checkGate.state === "failed" && validationRun) {
      failureDetails = await buildWorkflowFailureDetails(github, validationRun, "checks");
      message = failureMessageFromDetails(failureDetails);
      workflowRunUrl = validationRun.html_url;
    } else if (checkGate.state === "failed") {
      failureDetails = { stage: "checks", step: "start timeout" };
    }

    if (checkGate.state === "applying") {
      const applyRuns = await github.listWorkflowRuns(APPLY_WORKFLOW_ID, {
        branch: operation.branch,
        event: "pull_request",
        perPage: 5,
      });
      const applyRun = applyRuns.workflow_runs.find(
        (candidate) => candidate.head_sha === pr.head.sha,
      );
      const applyGate = decideApplyGate(operation, applyRun);
      nextState = applyGate.state;
      message = applyGate.message;
      if (applyRun) {
        workflowRunUrl = applyRun.html_url;
      }
      if (applyGate.state === "failed" && applyRun) {
        failureDetails = await buildWorkflowFailureDetails(github, applyRun, "apply");
        message = failureMessageFromDetails(failureDetails);
      } else if (applyGate.state === "failed") {
        failureDetails = { stage: "apply", step: "start timeout" };
      }

      if (applyGate.shouldAttemptMerge) {
        if (!allowMergeAttempt) {
          nextState = "pending_merge";
          message = buildOperationMessage(nextState);
        } else {
          const nodeLockGate = decideNodeLockGate(
            await claimNodeLockForMerge(env, github, operation),
          );
          nextState = nodeLockGate.state;
          message = nodeLockGate.message;

          if (nodeLockGate.shouldAttemptMerge) {
            const peerPath = PEER_FILE_PATH(operation.node);
            const [branchFile, baseFile] = await Promise.all([
              github.getFile(peerPath, operation.branch),
              github.getFile(peerPath, env.GITHUB_BASE_BRANCH),
            ]);

            if (branchFile.exists && baseFile.exists && branchFile.text === baseFile.text) {
              await github.closePullRequest(operation.pr_number);
              await github.deleteBranch(operation.branch).catch(() => {});
              nextState = "completed";
              message = buildOperationMessage(nextState);
              failureDetails = null;
            } else {
              try {
                await github.mergePullRequest(operation.pr_number, pr.head.sha);
                await github.deleteBranch(operation.branch).catch(() => {});
                nextState = "completed";
                message = buildOperationMessage(nextState);
                failureDetails = null;
              } catch (error) {
                await releaseNodeOperationLock(env, operation.node, operation.id);
                try {
                  if (branchFile.exists && branchFile.text) {
                    const currentBaseFile = await github.getFile(peerPath, env.GITHUB_BASE_BRANCH);
                    if (currentBaseFile.exists && currentBaseFile.text === branchFile.text) {
                      await github.closePullRequest(operation.pr_number);
                      await github.deleteBranch(operation.branch).catch(() => {});
                      nextState = "completed";
                      message = buildOperationMessage(nextState);
                      failureDetails = null;
                    } else {
                      const baseSha = await github.getBranchHead(env.GITHUB_BASE_BRANCH);
                      await github.forcePushSingleFile({
                        branch: operation.branch,
                        baseSha,
                        path: peerPath,
                        content: branchFile.text,
                        message: commitMessage(operation.kind, operation.asn, operation.node),
                      });
                      nextState = "pending_checks";
                      message = buildOperationMessage(nextState);
                      failureDetails = null;
                    }
                  } else {
                    nextState = "failed";
                    message = uiMessage("operation.message.merge_failed", {
                      error: error instanceof Error ? error.message : "unknown error",
                    });
                    failureDetails = {
                      stage: "merge",
                      step: "github merge",
                      conclusion: "merge_failed",
                      annotation: error instanceof Error ? error.message : "unknown error",
                    };
                  }
                } catch (rebaseError) {
                  nextState = "failed";
                  message = uiMessage("operation.message.merge_failed", {
                    error: rebaseError instanceof Error ? rebaseError.message : "unknown error",
                  });
                  failureDetails = {
                    stage: "merge",
                    step: "rebase",
                    conclusion: "merge_failed",
                    annotation: rebaseError instanceof Error ? rebaseError.message : "unknown error",
                  };
                }
              }
            }
          }
        }
      }
    }
  }

  const updated: OperationRecord = {
    ...operation,
    state: nextState,
    workflow_run_url: workflowRunUrl,
    message,
    failure_details: nextState === "failed" ? failureDetails : null,
    updated_at: nowIso(),
  };
  await putOperation(env, updated);
  if (isTerminalOperationState(updated.state)) {
    await releaseNodeOperationLock(env, updated.node, updated.id);
  }
  return updated;
}

async function listSessionsResponse(
  env: Env,
  request: Request,
  session: SessionRecord,
): Promise<Response> {
  const github = new GitHubClient(env);
  const [{ hosts, peerFiles }, existingOperations] = await Promise.all([
    loadRepoState(env, github),
    listOperationsForAsn(env, session.asn),
  ]);
  const operations = await Promise.all(
    existingOperations.map((operation) => refreshOperation(env, github, operation)),
  );

  const vaultPassword = readOptionalSecret(env, "ANSIBLE_VAULT_PASSWORD");
  const sessions = await listSessionsForAsn(session.asn, peerFiles, hosts, operations, vaultPassword, github);
  return jsonWithCors(request, {
    asn: session.asn,
    nodes: buildNodeViews(hosts),
    sessions,
  });
}

function findNodeOrThrow(name: string, nodes: Awaited<ReturnType<typeof loadRepoState>>["hosts"]) {
  const node = nodes.find((candidate) => candidate.name === name);
  if (!node) {
    throw new HttpError(uiMessage("error.node.not_eligible", { node: name }), 400);
  }
  return node;
}

async function handleMutation(
  env: Env,
  request: Request,
  kind: OperationRecord["kind"],
  nodeFromPath?: string,
): Promise<Response> {
  const authSession = await requireSession(env, request);
  if (!sessionCanMutate(env, authSession)) {
    throw new HttpError(
      uiMessage("error.auth.impersonation.host_asn.cannot_mutate", { asn: authSession.asn }),
      403,
    );
  }
  const github = new GitHubClient(env);
  const [repo, operations] = await Promise.all([
    loadRepoState(env, github),
    listOperationsForAsn(env, authSession.asn),
  ]);
  const vaultPassword = readOptionalSecret(env, "ANSIBLE_VAULT_PASSWORD");
  const sessions = await listSessionsForAsn(authSession.asn, repo.peerFiles, repo.hosts, operations, vaultPassword, github);

  let nodeName = nodeFromPath;
  let sessionPayload: CreateSessionRequest["session"] | UpdateSessionRequest["session"] | undefined;
  if (kind === "create") {
    const body = requireRecord(await readJson<CreateSessionRequest>(request), "request body");
    nodeName = requireNonEmptyString(body.node, "node");
    sessionPayload = parseRequestSessionSpec(body.session);
  } else if (kind === "update") {
    const body = requireRecord(await readJson<UpdateSessionRequest>(request), "request body");
    sessionPayload = parseRequestSessionSpec(body.session);
  }

  if (!nodeName) {
    throw new HttpError("error.request.node.required", 400);
  }

  const node = findNodeOrThrow(nodeName, repo.hosts);

  if (node.autopeer === false) {
    throw new HttpError(uiMessage("error.node.not_accepting_changes", { node: nodeName }), 403);
  }

  if (kind === "create") {
    if (!sessionPayload) {
      throw new HttpError("error.request.session_payload.required", 400);
    }
    if (sessions.some((session) => session.node === nodeName && session.state !== "manual")) {
      throw new HttpError(uiMessage("error.session.duplicate_on_node", { asn: authSession.asn, node: nodeName }), 409);
    }
    assertValidSessionSpec(node, authSession.asn, sessionPayload);
  }

  if (kind === "update" && sessionPayload) {
    assertValidSessionSpec(node, authSession.asn, sessionPayload);
  }

  if (sessionPayload && !vaultPassword && (sessionPayload.psk || sessionPayload.encrypt_endpoint)) {
    throw new HttpError("error.vault.not_configured", 501);
  }

  const peerPath = PEER_FILE_PATH(nodeName);
  const currentFile = await github.getFile(peerPath, env.GITHUB_BASE_BRANCH);
  if (!currentFile.exists || !currentFile.text || !currentFile.sha) {
    throw new HttpError(uiMessage("error.repo.peer_file.missing", { path: peerPath }), 502);
  }

  const mutationAuthMethod =
    kind === "migrate"
      ? {
        ...authSession.auth_method,
        provider: "migration",
      }
      : authSession.auth_method;

  let mutation;
  try {
    mutation = await mutatePeerFile(currentFile.text, {
      asn: authSession.asn,
      effectiveMnt: authSession.effective_mnt,
      authMethod: mutationAuthMethod,
      kind,
      session: sessionPayload,
      vaultPassword,
    });
  } catch (error) {
    throw new HttpError(errorMessage(error, "error.request.session_payload.invalid"), 409);
  }

  if (mutation.content === currentFile.text) {
    return jsonWithCors(
      request,
      buildNoChangeOperation(authSession, nodeName, kind, mutation.sessionSnapshot),
      200,
    );
  }

  const reusableOperation = await findReusableFailedOperation(
    env,
    github,
    operations,
    authSession.asn,
    nodeName,
  );

  if (reusableOperation) {
    const baseSha = await github.getBranchHead(env.GITHUB_BASE_BRANCH);
    await github.forcePushSingleFile({
      branch: reusableOperation.branch,
      baseSha,
      path: peerPath,
      content: mutation.content,
      message: commitMessage(kind, authSession.asn, nodeName),
    });

    const refreshedPr = reusableOperation.pr_number
      ? await github.getPullRequest(reusableOperation.pr_number)
      : null;

    const reusedAt = nowIso();
    const updated: OperationRecord = {
      ...reusableOperation,
      kind,
      state: "pending_checks",
      message: buildOperationMessage("pending_checks"),
      failure_details: null,
      workflow_run_url: null,
      pull_request_url: refreshedPr?.html_url ?? reusableOperation.pull_request_url ?? null,
      session_snapshot: mutation.sessionSnapshot,
      created_at: reusedAt,
      updated_at: reusedAt,
    };
    await putOperation(env, updated);
    return jsonWithCors(request, updated, 202);
  }

  const now = nowIso();
  const operation: OperationRecord = {
    id: crypto.randomUUID(),
    asn: authSession.asn,
    node: nodeName,
    kind,
    state: "pending_pull_request",
    branch: "",
    pr_number: null,
    pr_node_id: null,
    pull_request_url: null,
    workflow_run_url: null,
    message: buildOperationMessage("pending_pull_request"),
    failure_details: null,
    created_at: now,
    updated_at: now,
    session_snapshot: mutation.sessionSnapshot,
  };
  operation.branch = branchName(operation);

  const inserted = await insertOperation(env, operation);
  if (!inserted) {
    throw new HttpError(
      uiMessage("error.session.duplicate_on_node", { asn: authSession.asn, node: nodeName }),
      409,
    );
  }

  try {
    const baseSha = await github.getBranchHead(env.GITHUB_BASE_BRANCH);
    await github.createBranch(operation.branch, baseSha);
    await github.upsertFile({
      path: peerPath,
      branch: operation.branch,
      sha: currentFile.sha,
      content: mutation.content,
      message: commitMessage(kind, authSession.asn, nodeName),
    });

    const locale = resolveLocale(request);
    const authLabel = mutationAuthMethod.provider ?? mutationAuthMethod.kind;
    const prBodyEn = [
      t("en", "pr.body", { kind, asn: authSession.asn }),
      "",
      `- ${t("en", "pr.node")}: \`${nodeName}\``,
      `- ${t("en", "pr.maintainer")}: \`${authSession.effective_mnt}\``,
      `- ${t("en", "pr.auth")}: \`${authLabel}\``,
    ].join("\n");

    let prBody = prBodyEn;
    if (locale !== "en") {
      const localKind = t(locale, `kind.${kind}`);
      const prBodyLocal = [
        t(locale, "pr.body", { kind: localKind, asn: authSession.asn }),
        "",
        `- ${t(locale, "pr.node")}: \`${nodeName}\``,
        `- ${t(locale, "pr.maintainer")}: \`${authSession.effective_mnt}\``,
        `- ${t(locale, "pr.auth")}: \`${authLabel}\``,
      ].join("\n");
      prBody = `${prBodyLocal}\n\n---\n\n${prBodyEn}`;
    }

    const pr = await github.createPullRequest({
      title: `autopeer: ${kind} AS${authSession.asn} on ${nodeName}`,
      body: prBody,
      head: operation.branch,
      base: env.GITHUB_BASE_BRANCH,
    });

    operation.pr_number = pr.number;
    operation.pr_node_id = pr.node_id;
    operation.pull_request_url = pr.html_url;
    operation.state = "pending_checks";
    operation.message = buildOperationMessage(operation.state);
    operation.updated_at = nowIso();

    await putOperation(env, operation);
    return jsonWithCors(request, operation, 202);
  } catch (error) {
    await deleteOperation(env, operation.id);
    throw error;
  }
}

async function findReusableFailedOperation(
  env: Env,
  github: GitHubClient,
  operations: OperationRecord[],
  asn: string,
  node: string,
): Promise<OperationRecord | null> {
  const candidates = operations
    .filter((candidate) => candidate.asn === asn && candidate.node === node)
    .filter((candidate) => candidate.state === "failed" && candidate.pr_number && candidate.branch)
    .sort(
      (a, b) =>
        Date.parse(b.updated_at || b.created_at) - Date.parse(a.updated_at || a.created_at),
    );

  for (const candidate of candidates) {
    if (!candidate.pr_number) continue;
    try {
      const pr = await github.getPullRequest(candidate.pr_number);
      if (!pr.merged && pr.state === "open") {
        return candidate;
      }
    } catch (error) {
      console.warn("failed to inspect candidate PR for reuse", error);
    }
  }
  return null;
}

async function router(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);

  if (request.method === "OPTIONS") {
    return new Response(null, {
      status: 204,
      headers: buildCorsHeaders(request),
    });
  }

  if (request.method === "GET" && url.pathname === OPENAPI_PATH) {
    return jsonWithCors(request, openApiSpec(request, env), 200);
  }

  if (
    request.method === "GET" &&
    (url.pathname === SWAGGER_PATH || url.pathname === `${SWAGGER_PATH}/`)
  ) {
    return new Response(swaggerUiHtml(OPENAPI_PATH), {
      headers: {
        "content-type": "text/html; charset=utf-8",
        "cache-control": "no-store",
      },
    });
  }

  if (request.method === "GET" && url.pathname === CONFIG_PATH) {
    return jsonWithCors(request, runtimeConfigResponse(request, env));
  }

  if (request.method === "GET" && url.pathname === "/health") {
    return jsonWithCors(request, { ok: true, now: nowIso() });
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/start") {
    const body = requireRecord(await readJson<AuthStartRequest>(request), "request body");
    const asn = normalizeSupportedAutopeerAsn(requireNonEmptyString(body.asn, "asn"));
    const maintainers = await loadMaintainersForRequestAsn(env, asn);
    const challenge = createChallenge(asn);
    challenge.maintainers = maintainers;
    challenge.methods = methodsFromMaintainers(maintainers, [], {
      registryEmailEnabled: registryEmailAuthConfigured(env),
    });

    if (challenge.methods.length === 0) {
      const key = configuredOidcProviders(env).length > 0
        ? "error.auth.asn.no_registry_auth.oidc_hint"
        : "error.auth.asn.no_supported_auth";
      throw new HttpError(uiMessage(key, { asn }), 400);
    }

    await putChallenge(env, challenge);
    const response: AuthStartResponse = {
      asn,
      challenge_id: challenge.id,
      challenge_text: challenge.challenge_text,
      challenge_ttl_seconds: 15 * 60,
      methods: challenge.methods,
    };
    return jsonWithCors(request, response);
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/impersonate") {
    const impersonatorSession = await requireSession(env, request);
    if (!sessionCanImpersonate(env, impersonatorSession)) {
      throw new HttpError(
        uiMessage("error.auth.impersonation.asn.not_host", { asn: impersonatorSession.asn }),
        403,
      );
    }

    const body = requireRecord(
      await readJson<HostImpersonationRequest>(request),
      "request body",
    );
    const asn = normalizeSupportedAutopeerAsn(requireNonEmptyString(body.asn, "asn"));
    const maintainers = await loadMaintainersForRequestAsn(env, asn);
    const effectiveMnt = resolveEffectiveMaintainer(
      maintainers,
      requireOptionalString(body.effective_mnt, "effective_mnt"),
    );
    const createdAt = nowIso();
    const session: SessionRecord = {
      token: crypto.randomUUID(),
      asn,
      effective_mnt: effectiveMnt,
      auth_method: {
        kind: "host_impersonation",
        label: uiMessage("auth_method.host_impersonation.label"),
        description: uiMessage("auth_method.host_impersonation.description", {
          mnt: effectiveMnt,
          host_asn: impersonatorSession.asn,
        }),
        provider: `AS${impersonatorSession.asn}`,
      },
      created_at: createdAt,
      expires_at: new Date(Date.parse(createdAt) + 6 * 60 * 60 * 1000).toISOString(),
    };
    await putAuthSession(env, session);
    return jsonWithCors(request, authSessionResponseForEnv(env, session));
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/verify/registry-ssh") {
    const body = requireRecord(
      await readJson<RegistrySshVerifyRequest>(request),
      "request body",
    );
    const challengeId = requireNonEmptyString(body.challenge_id, "challenge_id");
    const verifyRequest: RegistrySshVerifyRequest = {
      challenge_id: challengeId,
      signature: body.signature as RegistrySshVerifyRequest["signature"],
    };
    const challenge = await consumeChallengeOrThrow(env, challengeId);
    const session = await verifyRegistrySshChallenge(challenge, verifyRequest);
    await putAuthSession(env, session);
    return jsonWithCors(request, authSessionResponseForEnv(env, session));
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/verify/registry-pgp") {
    const body = requireRecord(
      await readJson<RegistryPgpVerifyRequest>(request),
      "request body",
    );
    const challengeId = requireNonEmptyString(body.challenge_id, "challenge_id");
    const verifyRequest: RegistryPgpVerifyRequest = {
      challenge_id: challengeId,
      public_key: body.public_key as RegistryPgpVerifyRequest["public_key"],
      signed_message: body.signed_message as RegistryPgpVerifyRequest["signed_message"],
    };
    const challenge = await consumeChallengeOrThrow(env, challengeId);
    const session = await verifyRegistryPgpChallenge(challenge, verifyRequest);
    await putAuthSession(env, session);
    return jsonWithCors(request, authSessionResponseForEnv(env, session));
  }

  if (request.method === "GET" && url.pathname === "/v1/auth/lookup/pgp-key") {
    const rawFingerprint = url.searchParams.get("fingerprint") ?? "";
    const normalized = normalizePgpFingerprint(rawFingerprint);
    if (!normalized) {
      throw new HttpError("error.auth.pgp.invalid_fingerprint", 400);
    }
    const result = await lookupPgpKeyOnKeyservers(normalized);
    const response: PgpKeyLookupResponse = result.publicKey
      ? {
          fingerprint: result.fingerprint,
          found: true,
          public_key: result.publicKey,
          source: result.source ?? undefined,
        }
      : { fingerprint: result.fingerprint, found: false };
    return jsonWithCors(request, response);
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/verify/registry-email/send") {
    if (!registryEmailAuthConfigured(env)) {
      throw new HttpError("error.auth.registry_email.unavailable", 503);
    }
    const body = requireRecord(
      await readJson<RegistryEmailSendRequest>(request),
      "request body",
    );
    const challengeId = requireNonEmptyString(body.challenge_id, "challenge_id");
    const challenge = await getChallenge(env, challengeId);
    if (!challenge) {
      throw new HttpError("error.auth.challenge.unknown_id", 404);
    }
    assertChallengeFresh(challenge);

    const target = resolveRegistryEmailTarget(
      challenge,
      requireOptionalString(body.effective_mnt, "effective_mnt"),
    );
    const requestedLocale = requireOptionalString(body.locale, "locale");
    const locale = resolveLocaleCode(requestedLocale) ?? resolveLocale(request);
    const emailAuthRequest = createRegistryEmailAuthRequest(
      challenge,
      target.maintainer,
      target.emails,
      locale,
    );
    await sendRegistryEmailAuthMessage(
      env,
      locale,
      challenge.asn,
      target.maintainer,
      emailAuthRequest,
      registryEmailCallbackUrl(env, request, challenge.id, emailAuthRequest.token),
    );
    await putRegistryEmailAuthRequest(env, emailAuthRequest);

    const response: RegistryEmailSendResponse = {
      effective_mnt: target.maintainer,
      emails: target.emails,
      expires_at: emailAuthRequest.expires_at,
    };
    return jsonWithCors(request, response);
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/verify/registry-email") {
    const body = requireRecord(
      await readJson<RegistryEmailVerifyRequest>(request),
      "request body",
    );
    const challengeId = requireNonEmptyString(body.challenge_id, "challenge_id");
    const code = requireNonEmptyString(body.code, "code");
    const emailAuthRequest = await getRegistryEmailAuthRequest(env, challengeId);
    if (!emailAuthRequest) {
      throw new HttpError("error.auth.registry_email.state.missing", 404);
    }

    if (emailAuthRequest.session_token) {
      throw new HttpError("error.auth.registry_email.already_completed", 409);
    }

    if (isExpired(emailAuthRequest.expires_at)) {
      await deleteRegistryEmailAuthRequest(env, challengeId);
      throw new HttpError("error.auth.registry_email.state.expired", 400);
    }

    if (!timingSafeEqual(code, emailAuthRequest.code)) {
      throw new HttpError("error.auth.registry_email.code.invalid", 400);
    }

    const session = await createCompletedRegistryEmailSession(
      env,
      challengeId,
      emailAuthRequest.effective_mnt,
    );
    await deleteRegistryEmailAuthRequest(env, challengeId);
    return jsonWithCors(request, authSessionResponseForEnv(env, session));
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/verify/registry-email/complete") {
    const body = requireRecord(
      await readJson<RegistryEmailCompleteRequest>(request),
      "request body",
    );
    const token = requireNonEmptyString(body.token, "token");
    const emailAuthRequest = await consumeCompletedRegistryEmailAuthRequestByToken(env, token);
    if (!emailAuthRequest) {
      const pendingRequest = await getRegistryEmailAuthRequestByToken(env, token);
      if (!pendingRequest) {
        throw new HttpError("error.auth.registry_email.state.missing", 404);
      }
      if (isExpired(pendingRequest.expires_at)) {
        await deleteRegistryEmailAuthRequest(env, pendingRequest.challenge_id);
        throw new HttpError("error.auth.registry_email.state.expired", 400);
      }
      if (!pendingRequest.session_token) {
        throw new HttpError("error.auth.registry_email.state.pending", 409);
      }
      throw new HttpError("error.auth.registry_email.state.missing", 404);
    }

    const sessionToken = emailAuthRequest.session_token;
    if (!sessionToken) {
      throw new HttpError("error.auth.registry_email.state.missing", 404);
    }

    const session = await getAuthSession(env, sessionToken);
    if (!session) {
      throw new HttpError("error.auth.registry_email.session.missing", 404);
    }
    if (isExpired(session.expires_at)) {
      throw new HttpError("error.auth.registry_email.session.expired", 401);
    }
    return jsonWithCors(request, authSessionResponseForEnv(env, session));
  }

  if (
    request.method === "POST" &&
    url.pathname.startsWith("/v1/auth/oidc/") &&
    url.pathname.endsWith("/start") &&
    url.pathname.split("/").length === 6
  ) {
    const providerName = decodeURIComponent(url.pathname.split("/")[4] ?? "");
    const body = requireRecord(await readJson<OidcStartRequest>(request), "request body");
    const challengeId = requireOptionalString(body.challenge_id, "challenge_id");
    if (challengeId) {
      const challenge = await getChallenge(env, challengeId);
      if (!challenge) {
        throw new HttpError("error.auth.challenge.unknown_id", 404);
      }
      assertChallengeFresh(challenge);
    }

    const provider = oidcProviderByName(configuredOidcProviders(env), providerName);
    if (!provider) {
      throw new HttpError(uiMessage("error.auth.oidc.provider.unknown", { provider: providerName }), 404);
    }

    const discovery = await fetchOidcDiscovery(provider);
    if (isDn42Request(env, request) && provider.dn42_issuer) {
      discovery.authorization_endpoint = rewriteIssuerHost(
        discovery.authorization_endpoint,
        provider,
      );
    }
    const redirectUri = oidcCallbackUrl(env, request, providerName);
    const authorization = await createOidcAuthorizationRequest(
      provider,
      discovery,
      challengeId ?? "",
      redirectUri,
    );

    await putOidcAuthRequest(env, authorization.record);
    const response: OidcStartResponse = {
      authorization_url: authorization.authorizationUrl,
    };
    return jsonWithCors(request, response);
  }

  if (request.method === "GET" && url.pathname.startsWith(OIDC_CALLBACK_PREFIX)) {
    const providerName = decodeURIComponent(url.pathname.slice(OIDC_CALLBACK_PREFIX.length));
    if (!providerName) {
      throw new HttpError("error.auth.oidc.callback.provider.missing", 400);
    }

    const error = url.searchParams.get("error");
    if (error) {
      const description = url.searchParams.get("error_description");
      const message = uiMessage("error.auth.oidc.provider.rejected", {
        error,
        description: description ?? "",
      });
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("oidc_error", message),
      );
    }

    const state = url.searchParams.get("state");
    const code = url.searchParams.get("code");
    if (!state || !code) {
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("oidc_error", uiMessage("error.auth.oidc.callback.params.missing")),
      );
    }

    const provider = oidcProviderByName(configuredOidcProviders(env), providerName);
    if (!provider) {
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("oidc_error", uiMessage("error.auth.oidc.provider.unknown", { provider: providerName })),
      );
    }

    const authRequest = await getOidcAuthRequest(env, state);
    if (!authRequest || authRequest.provider !== providerName) {
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("oidc_error", uiMessage("error.auth.oidc.state.missing")),
      );
    }

    if (authRequest.session_token) {
      return siteRedirectResponse(
        env,
        request,
        `oidc_state=${encodeURIComponent(authRequest.state)}`,
      );
    }

    if (isExpired(authRequest.expires_at)) {
      await deleteOidcAuthRequest(env, authRequest.state);
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("oidc_error", uiMessage("error.auth.oidc.state.expired")),
      );
    }

    let challenge = null;
    if (authRequest.challenge_id) {
      challenge = await getChallenge(env, authRequest.challenge_id);
      if (!challenge) {
        await deleteOidcAuthRequest(env, authRequest.state);
        return siteRedirectResponse(
          env,
          request,
          fragmentMessage("oidc_error", uiMessage("error.auth.challenge.expired")),
        );
      }
    }

    try {
      if (challenge) {
        assertChallengeFresh(challenge);
      }
      const discovery = await fetchOidcDiscovery(provider);
      const tokenResponse = await exchangeAuthorizationCode(
        env,
        provider,
        discovery,
        code,
        authRequest.redirect_uri,
        authRequest.code_verifier,
      );
      const claimSources = await verifiedOidcClaimSources(
        tokenResponse,
        provider,
        discovery,
        authRequest.nonce,
      );
      const tokenAsn = normalizeSupportedAutopeerAsn(
        oidcAsnFromClaimSources(claimSources, provider),
      );
      let session: SessionRecord;

      if (challenge) {
        if (tokenAsn !== challenge.asn) {
          throw new HttpError(
            uiMessage("error.auth.oidc.identity.asn_mismatch", {
              token_asn: tokenAsn,
              requested_asn: challenge.asn,
            }),
            400,
          );
        }
        const effectiveMnt = oidcMaintainerFromClaimSources(
          claimSources,
          provider,
          challenge.maintainers,
        );
        session = sessionFromOidcIdentity(provider, challenge.asn, effectiveMnt);
      } else {
        const maintainers = await loadMaintainersForRequestAsn(env, tokenAsn);
        const effectiveMnt = oidcMaintainerFromClaimSources(
          claimSources,
          provider,
          maintainers,
        );
        session = sessionFromOidcIdentity(provider, tokenAsn, effectiveMnt);
      }

      await putAuthSession(env, session);
      await putOidcAuthRequest(env, {
        ...authRequest,
        session_token: session.token,
      });
      if (challenge) {
        await deleteChallenge(env, challenge.id);
      }
      return siteRedirectResponse(
        env,
        request,
        `oidc_state=${encodeURIComponent(authRequest.state)}`,
      );
    } catch (callbackError) {
      await deleteOidcAuthRequest(env, authRequest.state);
      const message = errorMessage(callbackError, "error.auth.oidc.callback.failed");
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("oidc_error", message),
      );
    }
  }

  if (request.method === "GET" && url.pathname === REGISTRY_EMAIL_CALLBACK_PATH) {
    const challengeId = url.searchParams.get("challenge_id");
    const token = url.searchParams.get("token");
    if (!challengeId || !token) {
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("email_error", uiMessage("error.auth.registry_email.callback.params.missing")),
      );
    }

    const emailAuthRequest = await getRegistryEmailAuthRequestByToken(env, token);
    if (!emailAuthRequest || emailAuthRequest.challenge_id !== challengeId) {
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("email_error", uiMessage("error.auth.registry_email.state.missing")),
      );
    }

    if (emailAuthRequest.session_token) {
      return siteRedirectResponse(
        env,
        request,
        withLangFragment(`email_token=${encodeURIComponent(emailAuthRequest.token)}`, emailAuthRequest.locale),
      );
    }

    if (isExpired(emailAuthRequest.expires_at)) {
      await deleteRegistryEmailAuthRequest(env, emailAuthRequest.challenge_id);
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("email_error", uiMessage("error.auth.registry_email.state.expired")),
      );
    }

    try {
      const session = await createCompletedRegistryEmailSession(
        env,
        challengeId,
        emailAuthRequest.effective_mnt,
      );
      await putRegistryEmailAuthRequest(env, {
        ...emailAuthRequest,
        session_token: session.token,
      });
      return siteRedirectResponse(
        env,
        request,
        withLangFragment(`email_token=${encodeURIComponent(emailAuthRequest.token)}`, emailAuthRequest.locale),
      );
    } catch (callbackError) {
      await deleteRegistryEmailAuthRequest(env, emailAuthRequest.challenge_id);
      const message = errorMessage(callbackError, "error.auth.registry_email.callback.failed");
      return siteRedirectResponse(
        env,
        request,
        fragmentMessage("email_error", message),
      );
    }
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/oidc/complete") {
    const body = requireRecord(await readJson<OidcCompleteRequest>(request), "request body");
    const authRequest = await getOidcAuthRequest(env, requireNonEmptyString(body.state, "state"));
    if (!authRequest) {
      throw new HttpError("error.auth.oidc.state.missing", 404);
    }
    if (!authRequest.session_token && isExpired(authRequest.expires_at)) {
      await deleteOidcAuthRequest(env, authRequest.state);
      throw new HttpError("error.auth.oidc.state.expired", 400);
    }
    if (!authRequest.session_token) {
      throw new HttpError("error.auth.oidc.state.pending", 409);
    }

    const session = await getAuthSession(env, authRequest.session_token);
    await deleteOidcAuthRequest(env, authRequest.state);
    if (!session) {
      throw new HttpError("error.auth.oidc.session.missing", 404);
    }
    if (isExpired(session.expires_at)) {
      throw new HttpError("error.auth.oidc.session.expired", 401);
    }

    return jsonWithCors(request, authSessionResponseForEnv(env, session));
  }

  if (request.method === "GET" && url.pathname === "/v1/sessions") {
    const session = await requireSession(env, request);
    return listSessionsResponse(env, request, session);
  }

  if (request.method === "POST" && url.pathname === "/v1/sessions") {
    return handleMutation(env, request, "create");
  }

  const sessionActionMatch = url.pathname.match(/^\/v1\/sessions\/([^/]+)\/([^/]+)\/(migrate|retire)$/);
  if (request.method === "POST" && sessionActionMatch) {
    const [, nodeName, asnPath, action] = sessionActionMatch;
    const session = await requireSession(env, request);
    if (normalizeSupportedAutopeerAsn(asnPath) !== session.asn) {
      throw new HttpError("error.auth.session.path_asn_mismatch", 403);
    }
    return handleMutation(env, request, action as OperationKind, nodeName);
  }

  const sessionPathMatch = url.pathname.match(/^\/v1\/sessions\/([^/]+)\/([^/]+)$/);
  if (sessionPathMatch) {
    const [, nodeName, asnPath] = sessionPathMatch;
    const session = await requireSession(env, request);
    if (normalizeSupportedAutopeerAsn(asnPath) !== session.asn) {
      throw new HttpError("error.auth.session.path_asn_mismatch", 403);
    }

    if (request.method === "PATCH") {
      return handleMutation(env, request, "update", nodeName);
    }
    if (request.method === "DELETE") {
      return handleMutation(env, request, "delete", nodeName);
    }
  }

  const operationMatch = url.pathname.match(/^\/v1\/operations\/([^/]+)$/);
  if (request.method === "GET" && operationMatch) {
    const { operation } = await requireOwnedOperation(env, request, operationMatch[1]);
    const github = new GitHubClient(env);
    const refreshed = await refreshOperation(env, github, operation);
    return jsonWithCors(request, refreshed);
  }

  const operationRetryMatch = url.pathname.match(/^\/v1\/operations\/([^/]+)\/retry$/);
  if (request.method === "POST" && operationRetryMatch) {
    const { operation } = await requireOwnedOperation(env, request, operationRetryMatch[1]);
    if (operation.state !== "failed" || !operation.pr_number || !operation.branch) {
      throw new HttpError("error.request.operation.not_retryable", 409);
    }

    const github = new GitHubClient(env);
    const pr = await github.getPullRequest(operation.pr_number);
    if (pr.merged || pr.state !== "open") {
      throw new HttpError("error.request.operation.pr_closed", 409);
    }

    const peerPath = PEER_FILE_PATH(operation.node);
    const branchFile = await github.getFile(peerPath, operation.branch);
    if (!branchFile.exists || !branchFile.text) {
      throw new HttpError("error.request.operation.branch_missing", 502);
    }

    const baseSha = await github.getBranchHead(env.GITHUB_BASE_BRANCH);
    await github.forcePushSingleFile({
      branch: operation.branch,
      baseSha,
      path: peerPath,
      content: branchFile.text,
      message: commitMessage(operation.kind, operation.asn, operation.node),
    });

    const retriedAt = nowIso();
    const updated: OperationRecord = {
      ...operation,
      state: "pending_checks",
      message: buildOperationMessage("pending_checks"),
      failure_details: null,
      workflow_run_url: null,
      created_at: retriedAt,
      updated_at: retriedAt,
    };
    await putOperation(env, updated);
    return jsonWithCors(request, updated, 202);
  }

  const operationDropMatch = url.pathname.match(/^\/v1\/operations\/([^/]+)\/drop$/);
  if (request.method === "POST" && operationDropMatch) {
    const { operation } = await requireOwnedOperation(env, request, operationDropMatch[1]);
    if (operation.state !== "failed" || !operation.pr_number) {
      throw new HttpError("error.request.operation.not_droppable", 409);
    }

    const github = new GitHubClient(env);
    const pr = await github.getPullRequest(operation.pr_number);
    if (pr.state === "open") {
      await github.closePullRequest(operation.pr_number);
    }
    if (operation.branch) {
      try {
        await github.deleteBranch(operation.branch);
      } catch {
        // branch may already be deleted
      }
    }

    const updated: OperationRecord = {
      ...operation,
      state: "completed",
      message: uiMessage("operation.message.dropped"),
      failure_details: null,
      updated_at: nowIso(),
    };
    await putOperation(env, updated);
    return jsonWithCors(request, updated, 200);
  }

  throw new HttpError("error.request.route.not_found", 404);
}

async function refreshActiveOperations(env: Env): Promise<void> {
  const operations = await listActiveOperations(env);
  if (operations.length === 0) return;

  const github = new GitHubClient(env);
  await Promise.all(
    operations.map((operation) =>
      refreshOperation(env, github, operation).catch((error) =>
        console.error(`cron: failed to refresh operation ${operation.id}`, error),
      ),
    ),
  );
}

export default {
  async fetch(request, env): Promise<Response> {
    try {
      return await router(request, env);
    } catch (error) {
      const rawMessage = error instanceof Error ? error.message : "internal error";
      if (error instanceof HttpError) {
        const publicMessage = isUiMessageKey(error.uiMessage.key)
          ? error.uiMessage
          : uiMessage(stripOperatorHints(error.uiMessage.key));
        return errorWithCors(request, publicMessage, error.status);
      }
      if (error instanceof I18nError) {
        return errorWithCors(request, error.uiMessage, 500);
      }

      console.error("autopeer-worker request failed", error);

      return errorWithCors(request, uiMessage(stripOperatorHints(rawMessage)), 500);
    }
  },

  async scheduled(_event, env, _ctx): Promise<void> {
    await refreshActiveOperations(env);
  },
} satisfies ExportedHandler<Env>;
