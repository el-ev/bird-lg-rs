import {
  assertChallengeFresh,
  createChallenge,
  createRegistryEmailAuthRequest,
  createRegistryEmailSession,
  verifyRegistryPgpChallenge,
  verifyRegistrySshChallenge,
} from "./auth";
import {
  claimNodeOperationLock,
  consumeFreshChallenge,
  consumeCompletedRegistryEmailAuthRequestByToken,
  deleteChallenge,
  deleteRegistryEmailAuthRequest,
  getAuthSession,
  getChallenge,
  getOidcAuthRequest,
  getRegistryEmailAuthRequest,
  getRegistryEmailAuthRequestByToken,
  getOperation,
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
import { sendRegistryEmailAuthMessage } from "./mailer";
import {
  buildNodeViews,
  listSessionsForAsn,
  loadInventoryHosts,
  mutatePeerFile,
  nodeIsAvailable,
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
  sessionFromOidcIdentity,
  verifiedOidcClaimSources,
} from "./oidc";
import { loadMaintainersForAsn, methodsFromMaintainers } from "./registry";
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
  OperationRecord,
  OperationState,
  OperationStatus,
  PeerSessionSpec,
  RegistryPgpVerifyRequest,
  RegistrySshVerifyRequest,
  SessionRecord,
  SessionView,
  UpdateSessionRequest,
} from "./types";
import {
  bearerToken,
  buildCorsHeaders,
  errorWithCors,
  HttpError,
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
  stripOperatorHints,
  timingSafeEqual,
} from "./utils";

const INVENTORY_PATH = "inventory.yaml";
const AUTOPEER_POLICY_PATH = "group_vars/all/autopeer.yaml";
const PEER_FILE_PATH = (node: string): string => `host_vars/${node}/dn42_peers.yaml`;
const CHECK_WORKFLOW_ID = "peer-session-check.yml";
const CHECK_WORKFLOW_GRACE_MS = 5 * 60 * 1000;
const APPLY_WORKFLOW_ID = "peer-session-apply.yml";
const APPLY_WORKFLOW_GRACE_MS = 10 * 60 * 1000;
const CONFIG_PATH = "/config.json";
const OIDC_CALLBACK_PREFIX = "/oidc/callback/";
const REGISTRY_EMAIL_CALLBACK_PATH = "/auth/email/callback";

type ValidationWorkflowRun = {
  status: string;
  conclusion: string | null;
};

type PreMergeGateDecision = {
  state: OperationState;
  message: string;
  shouldAttemptMerge: boolean;
};

type RefreshOperationOptions = {
  allowMergeAttempt?: boolean;
};

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error && error.message.trim().length > 0 ? error.message : fallback;
}

function requireRequestString(value: unknown, field: string): string {
  try {
    return requireNonEmptyString(value, field);
  } catch (error) {
    throw new HttpError(errorMessage(error, `${field} is required`), 400);
  }
}

function requireRequestRecord(value: unknown, field: string): Record<string, unknown> {
  try {
    return requireRecord(value, field);
  } catch (error) {
    throw new HttpError(errorMessage(error, `${field} must be an object`), 400);
  }
}

function requireOptionalRequestString(value: unknown, field: string): string | null {
  try {
    return requireOptionalString(value, field);
  } catch (error) {
    throw new HttpError(errorMessage(error, `${field} must be a string`), 400);
  }
}

function requireRequestBoolean(value: unknown, field: string): boolean {
  try {
    return requireBoolean(value, field);
  } catch (error) {
    throw new HttpError(errorMessage(error, `${field} must be a boolean`), 400);
  }
}

function requireOptionalRequestInteger(value: unknown, field: string): number | null {
  try {
    return requireOptionalInteger(value, field);
  } catch (error) {
    throw new HttpError(errorMessage(error, `${field} must be an integer`), 400);
  }
}

function requireRequestAsn(value: unknown): string {
  const raw = requireRequestString(value, "asn");
  try {
    return normalizeSupportedAutopeerAsn(raw);
  } catch (error) {
    throw new HttpError(
      errorMessage(error, "We do not support that ASN range yet. Right now Autopeer only supports 424242xxxx."),
      400,
    );
  }
}

function normalizeRequestAsn(value: string): string {
  try {
    return normalizeSupportedAutopeerAsn(value);
  } catch (error) {
    throw new HttpError(
      errorMessage(error, "We do not support that ASN range yet. Right now Autopeer only supports 424242xxxx."),
      400,
    );
  }
}

export function classifyMaintainerLookupError(asn: string, error: unknown): HttpError {
  const message = errorMessage(
    error,
    `DN42 registry lookup failed while loading AS${asn}`,
  );

  if (message === `Registry path not found: data/aut-num/AS${asn}`) {
    return new HttpError(
      `AS${asn} is invalid because it does not exist in the DN42 registry.`,
      400,
    );
  }

  if (message === `AS${asn} does not expose any mnt-by records in the registry`) {
    return new HttpError(
      `AS${asn} exists in DN42, but it does not publish maintainer auth we can use yet.`,
      400,
    );
  }

  return new HttpError(message, 502);
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
  const record = requireRequestRecord(value, "session");
  const peeringStrategy = requireOptionalRequestString(
    record.peering_strategy,
    "session.peering_strategy",
  ) ?? "full_table";
  return {
    comment: requireOptionalRequestString(record.comment, "session.comment"),
    endpoint: requireRequestString(record.endpoint, "session.endpoint"),
    wg_public_key: requireRequestString(record.wg_public_key, "session.wg_public_key"),
    port: requireOptionalRequestInteger(record.port, "session.port"),
    peer4: requireOptionalRequestString(record.peer4, "session.peer4"),
    peer6: requireOptionalRequestString(record.peer6, "session.peer6"),
    own6: requireOptionalRequestString(record.own6, "session.own6"),
    keepalive: requireOptionalRequestInteger(record.keepalive, "session.keepalive"),
    mtu: requireOptionalRequestInteger(record.mtu, "session.mtu"),
    ipv4: requireRequestBoolean(record.ipv4, "session.ipv4"),
    ipv6: requireRequestBoolean(record.ipv6, "session.ipv6"),
    extended_next_hop: requireRequestBoolean(
      record.extended_next_hop,
      "session.extended_next_hop",
    ),
    mp_bgp: requireRequestBoolean(record.mp_bgp, "session.mp_bgp"),
    peering_strategy: peeringStrategy as PeerSessionSpec["peering_strategy"],
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
    throw new HttpError(errorMessage(error, "session payload is invalid"), 400);
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

function externalSiteBaseUrl(env: Env, request: Request): URL {
  const base = new URL(configuredUrl(env.AUTOPEER_SITE_URL) ?? request.url);
  const forwardedHost = forwardedHeaderValue(request, "x-forwarded-host");
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

function runtimeConfigResponse(request: Request, env: Env) {
  const origin = new URL(request.url).origin;
  return {
    autopeer_url: configuredUrl(env.AUTOPEER_URL) ?? origin,
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

function requireRegistryEmailAuthConfigured(env: Env): void {
  if (!registryEmailAuthConfigured(env)) {
    throw new HttpError("Registry email login is not available in this deployment", 503);
  }
}

async function consumeChallengeOrThrow(env: Env, challengeId: string): Promise<ChallengeRecord> {
  const result = await consumeFreshChallenge(env, challengeId);
  switch (result.kind) {
    case "available":
      return result.challenge;
    case "missing":
      throw new HttpError("unknown challenge_id", 404);
    case "expired":
      throw new HttpError("challenge has expired", 400);
    case "consumed":
      throw new HttpError("challenge has already been used", 400);
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
      `AS${challenge.asn} does not expose any admin-c or tech-c email addresses we can use in the registry`,
      400,
    );
  }

  if (requestedMaintainer) {
    const requested = requestedMaintainer.trim().toUpperCase();
    const matched = targets.find((target) => target.maintainer.toUpperCase() === requested);
    if (!matched) {
      throw new HttpError(
        `${requested} does not have registry email contacts we can use for this ASN`,
        400,
      );
    }
    return matched;
  }

  if (targets.length === 1) {
    return targets[0];
  }

  throw new HttpError(
    "effective_mnt is required when your registry email auth covers multiple maintainers",
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

function configuredHostAsns(env: Env): Set<string> {
  return parseConfiguredAsns(env.HOST_ASNS);
}

function sessionCanImpersonate(env: Env, session: SessionRecord): boolean {
  return configuredHostAsns(env).has(session.asn);
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

function resolveEffectiveMaintainer(
  maintainers: MaintainerRecord[],
  requestedMaintainer?: string | null,
): string {
  if (maintainers.length === 0) {
    throw new HttpError("AS has no maintainers available for impersonation", 400);
  }

  if (requestedMaintainer) {
    const requested = requestedMaintainer.trim().toUpperCase();
    const matched = maintainers.find((maintainer) => maintainer.name.toUpperCase() === requested);
    if (!matched) {
      throw new HttpError(
        `${requested} is not present in aut-num -> mnt-by for this ASN`,
        400,
      );
    }
    return matched.name;
  }

  if (maintainers.length === 1) {
    return maintainers[0].name;
  }

  throw new HttpError(
    "effective_mnt is required when your target ASN has multiple maintainers",
    400,
  );
}

async function requireSession(env: Env, request: Request): Promise<SessionRecord> {
  const token = bearerToken(request);
  if (!token) {
    throw new HttpError("missing bearer session token", 401);
  }

  const session = await getAuthSession(env, token);
  if (!session) {
    throw new HttpError("unknown auth session", 401);
  }
  if (Date.parse(session.expires_at) <= Date.now()) {
    throw new HttpError("auth session has expired", 401);
  }

  return session;
}

async function loadRepoState(env: Env, github: GitHubClient) {
  const inventoryFile = await github.getFile(INVENTORY_PATH, env.GITHUB_BASE_BRANCH);
  if (!inventoryFile.exists || !inventoryFile.text) {
    throw new HttpError("network repo is missing inventory.yaml", 502);
  }

  const policyFile = await github.getFile(AUTOPEER_POLICY_PATH, env.GITHUB_BASE_BRANCH);
  const hosts = loadInventoryHosts(inventoryFile.text, policyFile.text ?? null);
  const peerFiles = new Map<string, string>();

  for (const host of hosts) {
    const file = await github.getFile(PEER_FILE_PATH(host.name), env.GITHUB_BASE_BRANCH);
    if (!file.exists || !file.text) {
      throw new HttpError(`network repo is missing ${PEER_FILE_PATH(host.name)}`, 502);
    }
    peerFiles.set(host.name, file.text);
  }

  return {
    hosts,
    peerFiles,
  };
}

function buildOperationMessage(state: OperationState): string {
  switch (state) {
    case "pending_pull_request":
      return "We are preparing your pull request.";
    case "pending_checks":
      return "Your pull request is open; waiting for peer-session-check.";
    case "applying":
      return "Checks passed; applying your session to the node for verification.";
    case "pending_merge":
      return "Apply succeeded on the node; waiting for merge.";
    case "completed":
      return "Your change was applied and merged successfully.";
    case "failed":
      return "Your change failed.";
    case "conflict":
      return "We could not apply your change because our repo conflicted.";
  }
}

function buildNodeLockWaitMessage(): string {
  return "Apply succeeded; waiting for another change on this node to finish merging.";
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
    }
  } catch (error) {
    console.warn("failed to read workflow jobs", error);
  }

  return details;
}

function failureMessageFromDetails(details: OperationFailureDetails): string {
  const stage =
    details.stage === "preflight"
      ? "Preflight"
      : details.stage === "checks"
      ? "peer-session-check"
      : details.stage === "merge"
      ? "Merge"
      : "peer-session-apply";
  const parts = [`${stage} failed with ${details.conclusion ?? "unknown"}`];
  if (details.step) parts.push(`(step: ${details.step})`);
  if (details.annotation) parts.push(`— ${details.annotation}`);
  return parts.join(" ");
}

function selectApplyWorkflowRun(
  runs: GitHubWorkflowRun[],
  pr: { head: { sha: string } },
): GitHubWorkflowRun | undefined {
  return runs.find((candidate) => candidate.head_sha === pr.head.sha);
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
    message: "Your session already matches our repo, so we did not open a pull request.",
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
        message: "peer-session-check did not start for your pull request.",
        shouldAttemptMerge: false,
      };
    }
    return {
      state: "pending_checks",
      message: "Your pull request is open; waiting for peer-session-check to start.",
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
      message: `peer-session-check finished with ${validationRun.conclusion ?? "unknown"}`,
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
        message: "peer-session-apply did not start for your pull request.",
        shouldAttemptMerge: false,
      };
    }
    return {
      state: "applying",
      message: "Checks passed; waiting for peer-session-apply to start.",
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
      message: `peer-session-apply finished with ${applyRun.conclusion ?? "unknown"}`,
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
    message: buildNodeLockWaitMessage(),
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
    nextState = "completed";
    message = buildOperationMessage(nextState);
    failureDetails = null;
  } else if (pr.state !== "open") {
    nextState = "failed";
    message = "Your pull request was closed before merge.";
  } else {
    const validationRuns = await github.listWorkflowRuns(CHECK_WORKFLOW_ID, {
      event: "pull_request",
      perPage: 20,
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
        event: "pull_request",
        perPage: 20,
      });
      const applyRun = selectApplyWorkflowRun(applyRuns.workflow_runs, pr);
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
            try {
              await github.mergePullRequest(operation.pr_number, pr.head.sha);
              nextState = "completed";
              message = buildOperationMessage(nextState);
              failureDetails = null;
            } catch (error) {
              await releaseNodeOperationLock(env, operation.node, operation.id);
              nextState = "pending_merge";
              message = `${buildOperationMessage(nextState)} Merge attempt failed: ${
                error instanceof Error ? error.message : "unknown error"
              }`;
              failureDetails = {
                stage: "merge",
                step: "github merge",
                conclusion: "merge_failed",
                annotation: error instanceof Error ? error.message : "unknown error",
              };
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
  const { hosts, peerFiles } = await loadRepoState(env, github);
  const existingOperations = await listOperationsForAsn(env, session.asn);
  const operations: OperationRecord[] = [];

  for (const operation of existingOperations) {
    operations.push(await refreshOperation(env, github, operation));
  }

  const sessions = listSessionsForAsn(session.asn, peerFiles, hosts, operations);
  return jsonWithCors(request, {
    asn: session.asn,
    effective_mnt: session.effective_mnt,
    auth_method: session.auth_method,
    nodes: buildNodeViews(hosts),
    sessions,
  });
}

function findNodeOrThrow(name: string, nodes: Awaited<ReturnType<typeof loadRepoState>>["hosts"]) {
  const node = nodes.find((candidate) => candidate.name === name);
  if (!node) {
    throw new HttpError(`node ${name} is not autopeer-eligible`, 400);
  }
  return node;
}

function sessionExistsOnNode(node: string, sessions: SessionView[]): boolean {
  return sessions.some((session) => session.node === node && session.state !== "manual");
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
      `AS${authSession.asn} is one of our host ASN sessions; impersonate the ASN you want to manage before opening or modifying sessions`,
      403,
    );
  }
  const github = new GitHubClient(env);
  const repo = await loadRepoState(env, github);
  const operations = await listOperationsForAsn(env, authSession.asn);
  const sessions = listSessionsForAsn(authSession.asn, repo.peerFiles, repo.hosts, operations);

  let nodeName = nodeFromPath;
  let sessionPayload: CreateSessionRequest["session"] | UpdateSessionRequest["session"] | undefined;
  if (kind === "create") {
    const body = requireRequestRecord(await readJson<CreateSessionRequest>(request), "request body");
    nodeName = requireRequestString(body.node, "node");
    sessionPayload = parseRequestSessionSpec(body.session);
  } else if (kind === "update") {
    const body = requireRequestRecord(await readJson<UpdateSessionRequest>(request), "request body");
    sessionPayload = parseRequestSessionSpec(body.session);
  }

  if (!nodeName) {
    throw new HttpError("node is required", 400);
  }

  const node = findNodeOrThrow(nodeName, repo.hosts);

  if (node.autopeer === false) {
    throw new HttpError(`${nodeName} is not accepting autopeer changes right now`, 403);
  }

  if (kind === "create") {
    if (!sessionPayload) {
      throw new HttpError("session payload is required", 400);
    }
    if (sessionExistsOnNode(nodeName, sessions)) {
      throw new HttpError(`ASN ${authSession.asn} already has a session or pending operation on ${nodeName}`, 409);
    }
    assertValidSessionSpec(node, authSession.asn, sessionPayload);
  }

  if (kind === "update" && sessionPayload) {
    assertValidSessionSpec(node, authSession.asn, sessionPayload);
  }

  const peerPath = PEER_FILE_PATH(nodeName);
  const currentFile = await github.getFile(peerPath, env.GITHUB_BASE_BRANCH);
  if (!currentFile.exists || !currentFile.text || !currentFile.sha) {
    throw new HttpError(`network repo is missing ${peerPath}`, 502);
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
    mutation = mutatePeerFile(currentFile.text, {
      asn: authSession.asn,
      effectiveMnt: authSession.effective_mnt,
      authMethod: mutationAuthMethod,
      kind,
      session: sessionPayload,
    });
  } catch (error) {
    throw new HttpError(String((error as Error).message), 409);
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
      message: `feat: autopeer ${kind} AS${authSession.asn} on ${nodeName}`,
    });

    const refreshedPr = reusableOperation.pr_number
      ? await github.getPullRequest(reusableOperation.pr_number)
      : null;

    const updated: OperationRecord = {
      ...reusableOperation,
      kind,
      state: "pending_checks",
      message: buildOperationMessage("pending_checks"),
      failure_details: null,
      workflow_run_url: null,
      pull_request_url: refreshedPr?.html_url ?? reusableOperation.pull_request_url ?? null,
      session_snapshot: mutation.sessionSnapshot,
      updated_at: nowIso(),
    };
    await putOperation(env, updated);
    return jsonWithCors(request, updated, 202);
  }

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
    created_at: nowIso(),
    updated_at: nowIso(),
    session_snapshot: mutation.sessionSnapshot,
  };
  operation.branch = branchName(operation);

  const baseSha = await github.getBranchHead(env.GITHUB_BASE_BRANCH);
  await github.createBranch(operation.branch, baseSha);
  await github.upsertFile({
    path: peerPath,
    branch: operation.branch,
    sha: currentFile.sha,
    content: mutation.content,
    message: `feat: autopeer ${kind} AS${authSession.asn} on ${nodeName}`,
  });

  const pr = await github.createPullRequest({
    title: `autopeer: ${kind} AS${authSession.asn} on ${nodeName}`,
    body: [
      `Autopeer ${kind} request for AS${authSession.asn}.`,
      "",
      `- Node: \`${nodeName}\``,
      `- Maintainer: \`${authSession.effective_mnt}\``,
      `- Auth: \`${mutationAuthMethod.provider ?? mutationAuthMethod.kind}\``,
    ].join("\n"),
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

  if (request.method === "GET" && url.pathname === CONFIG_PATH) {
    return jsonWithCors(request, runtimeConfigResponse(request, env));
  }

  if (request.method === "GET" && url.pathname === "/health") {
    return jsonWithCors(request, { ok: true, now: nowIso() });
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/start") {
    const body = requireRequestRecord(await readJson<AuthStartRequest>(request), "request body");
    const asn = requireRequestAsn(body.asn);
    const maintainers = await loadMaintainersForRequestAsn(env, asn);
    const challenge = createChallenge(asn);
    challenge.maintainers = maintainers;
    challenge.methods = methodsFromMaintainers(maintainers, [], {
      registryEmailEnabled: registryEmailAuthConfigured(env),
    });

    if (challenge.methods.length === 0) {
      const message = configuredOidcProviders(env).length > 0
        ? `AS${asn} does not expose supported registry SSH, PGP, or email auth methods. Use one of the configured OIDC login options instead.`
        : `AS${asn} does not expose any supported auth methods you can use in the registry`;
      throw new HttpError(message, 400);
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
        `AS${impersonatorSession.asn} is not configured as a host ASN for impersonation`,
        403,
      );
    }

    const body = requireRequestRecord(
      await readJson<HostImpersonationRequest>(request),
      "request body",
    );
    const asn = requireRequestAsn(body.asn);
    const maintainers = await loadMaintainersForRequestAsn(env, asn);
    const effectiveMnt = resolveEffectiveMaintainer(
      maintainers,
      requireOptionalRequestString(body.effective_mnt, "effective_mnt"),
    );
    const createdAt = nowIso();
    const session: SessionRecord = {
      token: crypto.randomUUID(),
      asn,
      effective_mnt: effectiveMnt,
      auth_method: {
        kind: "host_impersonation",
        label: "Host ASN Impersonation",
        description: `You are impersonating ${effectiveMnt} through our host ASN.`,
        provider: `AS${impersonatorSession.asn}`,
      },
      created_at: createdAt,
      expires_at: new Date(Date.parse(createdAt) + 6 * 60 * 60 * 1000).toISOString(),
    };
    await putAuthSession(env, session);
    return jsonWithCors(request, authSessionResponseForEnv(env, session));
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/verify/registry-ssh") {
    const body = requireRequestRecord(
      await readJson<RegistrySshVerifyRequest>(request),
      "request body",
    );
    const challengeId = requireRequestString(body.challenge_id, "challenge_id");
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
    const body = requireRequestRecord(
      await readJson<RegistryPgpVerifyRequest>(request),
      "request body",
    );
    const challengeId = requireRequestString(body.challenge_id, "challenge_id");
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

  if (request.method === "POST" && url.pathname === "/v1/auth/verify/registry-email/send") {
    requireRegistryEmailAuthConfigured(env);
    const body = requireRequestRecord(
      await readJson<RegistryEmailSendRequest>(request),
      "request body",
    );
    const challengeId = requireRequestString(body.challenge_id, "challenge_id");
    const challenge = await getChallenge(env, challengeId);
    if (!challenge) {
      throw new HttpError("unknown challenge_id", 404);
    }
    assertChallengeFresh(challenge);

    const target = resolveRegistryEmailTarget(
      challenge,
      requireOptionalRequestString(body.effective_mnt, "effective_mnt"),
    );
    const emailAuthRequest = createRegistryEmailAuthRequest(
      challenge,
      target.maintainer,
      target.emails,
    );
    await sendRegistryEmailAuthMessage(
      env,
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
    const body = requireRequestRecord(
      await readJson<RegistryEmailVerifyRequest>(request),
      "request body",
    );
    const challengeId = requireRequestString(body.challenge_id, "challenge_id");
    const code = requireRequestString(body.code, "code");
    const emailAuthRequest = await getRegistryEmailAuthRequest(env, challengeId);
    if (!emailAuthRequest) {
      throw new HttpError("Registry email login state was not found or has expired", 404);
    }

    if (emailAuthRequest.session_token) {
      throw new HttpError(
        "Registry email login has already completed; finish it from the emailed sign-in link.",
        409,
      );
    }

    if (Date.parse(emailAuthRequest.expires_at) <= Date.now()) {
      await deleteRegistryEmailAuthRequest(env, challengeId);
      throw new HttpError("Registry email login has expired", 400);
    }

    if (!timingSafeEqual(code, emailAuthRequest.code)) {
      throw new HttpError("Registry email auth code is invalid", 400);
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
    const body = requireRequestRecord(
      await readJson<RegistryEmailCompleteRequest>(request),
      "request body",
    );
    const token = requireRequestString(body.token, "token");
    const emailAuthRequest = await consumeCompletedRegistryEmailAuthRequestByToken(env, token);
    if (!emailAuthRequest) {
      const pendingRequest = await getRegistryEmailAuthRequestByToken(env, token);
      if (!pendingRequest) {
        throw new HttpError("Registry email login state was not found or has expired", 404);
      }
      if (Date.parse(pendingRequest.expires_at) <= Date.now()) {
        await deleteRegistryEmailAuthRequest(env, pendingRequest.challenge_id);
        throw new HttpError("Registry email login has expired", 400);
      }
      if (!pendingRequest.session_token) {
        throw new HttpError("Registry email login has not completed yet", 409);
      }
      throw new HttpError("Registry email login state was not found or has expired", 404);
    }

    const sessionToken = emailAuthRequest.session_token;
    if (!sessionToken) {
      throw new HttpError("Registry email login state was not found or has expired", 404);
    }

    const session = await getAuthSession(env, sessionToken);
    if (!session) {
      throw new HttpError("Registry email login session is no longer available", 404);
    }
    if (Date.parse(session.expires_at) <= Date.now()) {
      throw new HttpError("Registry email login session has expired", 401);
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
    const body = requireRequestRecord(await readJson<OidcStartRequest>(request), "request body");
    const challengeId = requireOptionalRequestString(body.challenge_id, "challenge_id");
    if (challengeId) {
      const challenge = await getChallenge(env, challengeId);
      if (!challenge) {
        throw new HttpError("unknown challenge_id", 404);
      }
      assertChallengeFresh(challenge);
    }

    const provider = oidcProviderByName(configuredOidcProviders(env), providerName);
    if (!provider) {
      throw new HttpError(`unknown OIDC provider ${providerName}`, 404);
    }

    const discovery = await fetchOidcDiscovery(provider);
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
      throw new HttpError("OIDC provider is missing from the callback path", 400);
    }

    const error = url.searchParams.get("error");
    if (error) {
      const description = url.searchParams.get("error_description");
      const message = description ? `${error}: ${description}` : error;
      return siteRedirectResponse(
        env,
        request,
        `oidc_error=${encodeURIComponent(message)}`,
      );
    }

    const state = url.searchParams.get("state");
    const code = url.searchParams.get("code");
    if (!state || !code) {
      return siteRedirectResponse(
        env,
        request,
        "oidc_error=Missing%20OIDC%20callback%20parameters",
      );
    }

    const provider = oidcProviderByName(configuredOidcProviders(env), providerName);
    if (!provider) {
      return siteRedirectResponse(
        env,
        request,
        `oidc_error=${encodeURIComponent(`Unknown OIDC provider ${providerName}`)}`,
      );
    }

    const authRequest = await getOidcAuthRequest(env, state);
    if (!authRequest || authRequest.provider !== providerName) {
      return siteRedirectResponse(
        env,
        request,
        "oidc_error=OIDC%20login%20state%20was%20not%20found%20or%20has%20expired",
      );
    }

    if (authRequest.session_token) {
      return siteRedirectResponse(
        env,
        request,
        `oidc_state=${encodeURIComponent(authRequest.state)}`,
      );
    }

    if (Date.parse(authRequest.expires_at) <= Date.now()) {
      await deleteOidcAuthRequest(env, authRequest.state);
      return siteRedirectResponse(
        env,
        request,
        "oidc_error=OIDC%20login%20state%20has%20expired",
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
          "oidc_error=Your%20authentication%20challenge%20has%20expired.%20Start%20again.",
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
            `OIDC identity ASN ${tokenAsn} does not match requested ASN ${challenge.asn}`,
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
      const message = errorMessage(
        callbackError,
        "OIDC login failed; please try again.",
      );
      return siteRedirectResponse(
        env,
        request,
        `oidc_error=${encodeURIComponent(message)}`,
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
        "email_error=Missing%20registry%20email%20callback%20parameters",
      );
    }

    const emailAuthRequest = await getRegistryEmailAuthRequestByToken(env, token);
    if (!emailAuthRequest || emailAuthRequest.challenge_id !== challengeId) {
      return siteRedirectResponse(
        env,
        request,
        "email_error=Registry%20email%20login%20state%20was%20not%20found%20or%20has%20expired",
      );
    }

    if (emailAuthRequest.session_token) {
      return siteRedirectResponse(
        env,
        request,
        `email_token=${encodeURIComponent(emailAuthRequest.token)}`,
      );
    }

    if (Date.parse(emailAuthRequest.expires_at) <= Date.now()) {
      await deleteRegistryEmailAuthRequest(env, emailAuthRequest.challenge_id);
      return siteRedirectResponse(
        env,
        request,
        "email_error=Registry%20email%20login%20has%20expired",
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
        `email_token=${encodeURIComponent(emailAuthRequest.token)}`,
      );
    } catch (callbackError) {
      await deleteRegistryEmailAuthRequest(env, emailAuthRequest.challenge_id);
      const message = errorMessage(
        callbackError,
        "Registry email login failed; please try again.",
      );
      return siteRedirectResponse(
        env,
        request,
        `email_error=${encodeURIComponent(message)}`,
      );
    }
  }

  if (request.method === "POST" && url.pathname === "/v1/auth/oidc/complete") {
    const body = requireRequestRecord(await readJson<OidcCompleteRequest>(request), "request body");
    const authRequest = await getOidcAuthRequest(env, requireRequestString(body.state, "state"));
    if (!authRequest) {
      throw new HttpError("OIDC login state was not found or has expired", 404);
    }
    if (!authRequest.session_token && Date.parse(authRequest.expires_at) <= Date.now()) {
      await deleteOidcAuthRequest(env, authRequest.state);
      throw new HttpError("OIDC login state has expired", 400);
    }
    if (!authRequest.session_token) {
      throw new HttpError("OIDC login has not completed yet", 409);
    }

    const session = await getAuthSession(env, authRequest.session_token);
    await deleteOidcAuthRequest(env, authRequest.state);
    if (!session) {
      throw new HttpError("OIDC login session is no longer available", 404);
    }
    if (Date.parse(session.expires_at) <= Date.now()) {
      throw new HttpError("OIDC login session has expired", 401);
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

  const sessionMigrationMatch = url.pathname.match(/^\/v1\/sessions\/([^/]+)\/([^/]+)\/migrate$/);
  if (request.method === "POST" && sessionMigrationMatch) {
    const [, nodeName, asnPath] = sessionMigrationMatch;
    const session = await requireSession(env, request);
    if (normalizeRequestAsn(asnPath) !== session.asn) {
      throw new HttpError("The path ASN does not match your authenticated session", 403);
    }
    return handleMutation(env, request, "migrate", nodeName);
  }

  const sessionPathMatch = url.pathname.match(/^\/v1\/sessions\/([^/]+)\/([^/]+)$/);
  if (sessionPathMatch) {
    const [, nodeName, asnPath] = sessionPathMatch;
    const session = await requireSession(env, request);
    if (normalizeRequestAsn(asnPath) !== session.asn) {
      throw new HttpError("The path ASN does not match your authenticated session", 403);
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
    const authSession = await requireSession(env, request);
    const operation = await getOperation(env, operationMatch[1]);
    if (!operation || operation.asn !== authSession.asn) {
      throw new HttpError("operation not found", 404);
    }

    const github = new GitHubClient(env);
    const refreshed = await refreshOperation(env, github, operation);
    return jsonWithCors(request, refreshed);
  }

  throw new HttpError("not found", 404);
}

export default {
  async fetch(request, env): Promise<Response> {
    try {
      return await router(request, env);
    } catch (error) {
      const rawMessage = error instanceof Error ? error.message : "internal error";
      const publicMessage = stripOperatorHints(rawMessage);

      if (error instanceof HttpError) {
        return errorWithCors(request, publicMessage, error.status);
      }

      console.error("autopeer-worker request failed", error);

      return errorWithCors(request, publicMessage, 500);
    }
  },
} satisfies ExportedHandler<Env>;
