import {
  assertChallengeFresh,
  createChallenge,
  verifyRegistryPgpChallenge,
  verifyRegistrySshChallenge,
} from "./auth";
import {
  claimNodeOperationLock,
  consumeFreshChallenge,
  deleteChallenge,
  getAuthSession,
  getChallenge,
  getOidcAuthRequest,
  getOperation,
  listOperationsForAsn,
  deleteOidcAuthRequest,
  putAuthSession,
  putChallenge,
  putOidcAuthRequest,
  putOperation,
  releaseNodeOperationLock,
} from "./db";
import { branchName, GitHubClient } from "./github";
import type { GitHubWorkflowRun } from "./github";
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
  readNonNegativeIntegerEnv,
  requireBoolean,
  requireNonEmptyString,
  requireOptionalInteger,
  requireOptionalString,
  requireRecord,
  stripOperatorHints,
} from "./utils";
import { parseDocument } from "yaml";

const INVENTORY_PATH = "inventory.yaml";
const AUTOPEER_POLICY_PATH = "group_vars/all/autopeer.yaml";
const COMMON_VARS_PATH = "group_vars/all/common.yaml";
const PEER_FILE_PATH = (node: string): string => `host_vars/${node}/dn42_peers.yaml`;
const CHECK_WORKFLOW_ID = "peer-session-check.yml";
const CHECK_WORKFLOW_GRACE_MS = 2 * 60 * 1000;
const APPLY_WORKFLOW_ID = "peer-session-apply.yml";
const APPLY_WORKFLOW_GRACE_MS = 2 * 60 * 1000;
const DEFAULT_APPLY_RETRY_LIMIT = 2;
const DEFAULT_APPLY_RETRY_INTERVAL_SECONDS = 30;
const DEFAULT_DN42_TARGET_PREFIX = "dn42_";
const CONFIG_PATH = "/config.json";
const OIDC_CALLBACK_PREFIX = "/oidc/callback/";

type ValidationWorkflowRun = {
  status: string;
  conclusion: string | null;
};

type PreMergeCheckGateDecision = {
  state: OperationState;
  message: string;
  shouldAttemptMerge: boolean;
};

type RefreshOperationOptions = {
  allowMergeAttempt?: boolean;
};

type ApplyRetryDecision =
  | { action: "wait"; message: string }
  | { action: "dispatch"; message: string; attempt: number }
  | { action: "fail"; message: string };

type ApplyRunSelection = {
  run?: GitHubWorkflowRun;
  awaitingRetryRun: boolean;
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
    case "pending_merge":
      return "Your checks passed; waiting for merge.";
    case "merged":
      return "Your pull request merged; waiting for peer-session-apply.";
    case "applying":
      return "We are applying your session from our repo.";
    case "completed":
      return "Your change was applied successfully.";
    case "failed":
      return "Your change failed.";
    case "conflict":
      return "We could not apply your change because our repo conflicted.";
  }
}

function buildNodeLockWaitMessage(): string {
  return "Your checks passed; waiting for another change on this node to finish applying.";
}

function configuredApplyRetryLimit(env: Env): number {
  return readNonNegativeIntegerEnv(
    env,
    "AUTOPEER_APPLY_RETRY_LIMIT",
    DEFAULT_APPLY_RETRY_LIMIT,
  );
}

function configuredApplyRetryIntervalMs(env: Env): number {
  return (
    readNonNegativeIntegerEnv(
      env,
      "AUTOPEER_APPLY_RETRY_INTERVAL_SECONDS",
      DEFAULT_APPLY_RETRY_INTERVAL_SECONDS,
    ) * 1000
  );
}

function parseTimestamp(value: string | null | undefined): number {
  const parsed = Date.parse(value ?? "");
  return Number.isFinite(parsed) ? parsed : NaN;
}

function firstFiniteTimestamp(...values: Array<string | null | undefined>): number {
  for (const value of values) {
    const parsed = parseTimestamp(value);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }
  return NaN;
}

function applyRetryCount(operation: Pick<OperationRecord, "apply_retry_count">): number {
  return typeof operation.apply_retry_count === "number" && Number.isFinite(operation.apply_retry_count)
    ? operation.apply_retry_count
    : 0;
}

function buildApplyRetryWaitMessage(
  conclusion: string | null,
  attempt: number,
  limit: number,
  waitSeconds: number,
): string {
  return `peer-session-apply finished for your change with ${
    conclusion ?? "unknown"
  }; retry ${attempt}/${limit} in ${waitSeconds}s.`;
}

function buildApplyRetryDispatchMessage(
  conclusion: string | null,
  attempt: number,
  limit: number,
): string {
  return `peer-session-apply finished for your change with ${
    conclusion ?? "unknown"
  }; starting retry ${attempt}/${limit}.`;
}

function buildApplyRetryStartWaitMessage(attempt: number, limit: number): string {
  return `Retry ${attempt}/${limit} was queued; waiting for peer-session-apply to start.`;
}

function buildApplyRetryExhaustedMessage(conclusion: string | null, limit: number): string {
  if (limit === 0) {
    return `peer-session-apply finished for your change with ${conclusion ?? "unknown"}`;
  }
  return `peer-session-apply finished for your change with ${
    conclusion ?? "unknown"
  } after ${limit} retries.`;
}

function buildApplyRetryStartTimeoutMessage(attempt: number, limit: number): string {
  return `Retry ${attempt}/${limit} did not start peer-session-apply in time.`;
}

export function resolveDn42TargetPrefix(commonVarsText: string | null | undefined): string {
  if (!commonVarsText) {
    return DEFAULT_DN42_TARGET_PREFIX;
  }

  try {
    const value = parseDocument(commonVarsText).toJS();
    if (value && typeof value === "object" && !Array.isArray(value)) {
      const prefix = (value as Record<string, unknown>).dn42_prefix;
      if (typeof prefix === "string" && prefix.trim().length > 0) {
        return prefix;
      }
    }
  } catch {
    return DEFAULT_DN42_TARGET_PREFIX;
  }

  return DEFAULT_DN42_TARGET_PREFIX;
}

function dn42TargetName(asn: string, prefix: string): string {
  return `${prefix}${asn.slice(-4)}`;
}

export function buildApplyWorkflowDispatchInputs(
  operation: Pick<OperationRecord, "asn" | "node">,
  dn42TargetPrefix = DEFAULT_DN42_TARGET_PREFIX,
): Record<string, string> {
  return {
    deploy_host: operation.node,
    peer_targets: JSON.stringify([dn42TargetName(operation.asn, dn42TargetPrefix)]),
  };
}

async function dispatchApplyWorkflow(
  env: Env,
  github: GitHubClient,
  operation: Pick<OperationRecord, "asn" | "node">,
): Promise<void> {
  const commonVarsFile = await github.getFile(COMMON_VARS_PATH, env.GITHUB_BASE_BRANCH);
  const dn42TargetPrefix = resolveDn42TargetPrefix(commonVarsFile.text ?? null);

  await github.dispatchWorkflow(APPLY_WORKFLOW_ID, {
    ref: env.GITHUB_BASE_BRANCH,
    inputs: buildApplyWorkflowDispatchInputs(operation, dn42TargetPrefix),
  });
}

function selectApplyWorkflowRun(
  runs: GitHubWorkflowRun[],
  operation: Pick<OperationRecord, "last_apply_retry_at">,
  pr: {
    merge_commit_sha: string | null;
    head: { sha: string };
  },
): ApplyRunSelection {
  const retryQueuedAt = parseTimestamp(operation.last_apply_retry_at);
  if (Number.isFinite(retryQueuedAt)) {
    const retryRun = runs.find((candidate) => parseTimestamp(candidate.created_at) >= retryQueuedAt);
    if (retryRun) {
      return {
        run: retryRun,
        awaitingRetryRun: false,
      };
    }
    return {
      run: undefined,
      awaitingRetryRun: true,
    };
  }

  return {
    run: runs.find(
      (candidate) =>
        candidate.head_sha === pr.merge_commit_sha || candidate.head_sha === pr.head.sha,
    ),
    awaitingRetryRun: false,
  };
}

export function decideApplyFailureRetry(
  operation: Pick<OperationRecord, "apply_retry_count">,
  run: Pick<GitHubWorkflowRun, "conclusion" | "updated_at" | "created_at">,
  config: {
    retryLimit: number;
    retryIntervalMs: number;
  },
  now = Date.now(),
): ApplyRetryDecision {
  const retryCount = applyRetryCount(operation);
  if (retryCount >= config.retryLimit) {
    return {
      action: "fail",
      message: buildApplyRetryExhaustedMessage(run.conclusion, config.retryLimit),
    };
  }

  const nextAttempt = retryCount + 1;
  const retryAfter = firstFiniteTimestamp(run.updated_at, run.created_at);
  if (Number.isFinite(retryAfter)) {
    const waitMs = retryAfter + config.retryIntervalMs - now;
    if (waitMs > 0) {
      return {
        action: "wait",
        message: buildApplyRetryWaitMessage(
          run.conclusion,
          nextAttempt,
          config.retryLimit,
          Math.max(1, Math.ceil(waitMs / 1000)),
        ),
      };
    }
  }

  return {
    action: "dispatch",
    message: buildApplyRetryDispatchMessage(run.conclusion, nextAttempt, config.retryLimit),
    attempt: nextAttempt,
  };
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
    created_at: now,
    updated_at: now,
    apply_retry_count: 0,
    last_apply_retry_at: null,
    session_snapshot: sessionSnapshot,
  };
}

export function decidePreMergeCheckGate(
  operation: Pick<OperationRecord, "created_at">,
  validationRun: ValidationWorkflowRun | undefined,
  now = Date.now(),
): PreMergeCheckGateDecision {
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
    state: "pending_merge",
    message: buildOperationMessage("pending_merge"),
    shouldAttemptMerge: true,
  };
}

export function decideNodeLockGate(hasNodeLock: boolean): PreMergeCheckGateDecision {
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
      (ownerOperation.state === "merged" || ownerOperation.state === "applying") &&
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
  let nextApplyRetryCount = applyRetryCount(operation);
  let lastApplyRetryAt = operation.last_apply_retry_at ?? null;

  if (!pr.merged && pr.state !== "open") {
    nextState = "failed";
    message = "Your pull request was closed before merge.";
  } else if (!pr.merged) {
    const validationRuns = await github.listWorkflowRuns(CHECK_WORKFLOW_ID, {
      event: "pull_request",
      perPage: 20,
    });
    const validationRun = validationRuns.workflow_runs.find(
      (candidate) => candidate.head_sha === pr.head.sha,
    );
    const gate = decidePreMergeCheckGate(operation, validationRun);

    nextState = gate.state;
    message = gate.message;

    if (gate.shouldAttemptMerge) {
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
            nextState = "merged";
            message = buildOperationMessage(nextState);
          } catch (error) {
            await releaseNodeOperationLock(env, operation.node, operation.id);
            nextState = "pending_merge";
            message = `${buildOperationMessage(nextState)} Merge attempt failed: ${
              error instanceof Error ? error.message : "unknown error"
            }`;
          }
        }
      }
    }
  } else {
    const retryLimit = configuredApplyRetryLimit(env);
    const retryIntervalMs = configuredApplyRetryIntervalMs(env);
    const runs = await github.listWorkflowRuns(APPLY_WORKFLOW_ID, {
      branch: env.GITHUB_BASE_BRANCH,
      perPage: 50,
    });
    const selection = selectApplyWorkflowRun(runs.workflow_runs, operation, pr);
    const run = selection.run;

    if (!run) {
      if (selection.awaitingRetryRun) {
        const queuedAt = parseTimestamp(lastApplyRetryAt);
        if (Number.isFinite(queuedAt) && Date.now() - queuedAt > APPLY_WORKFLOW_GRACE_MS) {
          if (nextApplyRetryCount >= retryLimit) {
            nextState = "failed";
            message = buildApplyRetryStartTimeoutMessage(nextApplyRetryCount, retryLimit);
          } else {
            const nextAttempt = nextApplyRetryCount + 1;
            const retryQueuedNow = nowIso();
            try {
              await dispatchApplyWorkflow(env, github, operation);
              nextApplyRetryCount = nextAttempt;
              lastApplyRetryAt = retryQueuedNow;
              nextState = "merged";
              workflowRunUrl = null;
              message = `Retry ${nextAttempt}/${retryLimit} was queued after the previous retry did not start.`;
            } catch (error) {
              nextApplyRetryCount = nextAttempt;
              lastApplyRetryAt = retryQueuedNow;
              nextState = "merged";
              workflowRunUrl = null;
              message = `Retry ${nextAttempt}/${retryLimit} dispatch failed: ${
                error instanceof Error ? error.message : "unknown error"
              }`;
            }
          }
        } else {
          nextState = "merged";
          message = buildApplyRetryStartWaitMessage(nextApplyRetryCount, retryLimit);
        }
      } else {
        const mergedAt = pr.merged_at ? Date.parse(pr.merged_at) : NaN;
        if (Number.isFinite(mergedAt) && Date.now() - mergedAt > APPLY_WORKFLOW_GRACE_MS) {
          nextState = "failed";
          message = "peer-session-apply did not start for your merged commit.";
        } else {
          nextState = "merged";
          message = buildOperationMessage(nextState);
        }
      }
    } else {
      workflowRunUrl = run.html_url;
      if (run.status !== "completed") {
        nextState = "applying";
        message = buildOperationMessage(nextState);
      } else if (run.conclusion === "success") {
        nextState = "completed";
        message = buildOperationMessage(nextState);
      } else {
        const retryDecision = decideApplyFailureRetry(
          operation,
          run,
          {
            retryLimit,
            retryIntervalMs,
          },
          Date.now(),
        );
        if (retryDecision.action === "fail") {
          nextState = "failed";
          message = retryDecision.message;
        } else if (retryDecision.action === "wait") {
          nextState = "merged";
          message = retryDecision.message;
        } else {
          const retryQueuedNow = nowIso();
          try {
            await dispatchApplyWorkflow(env, github, operation);
            nextApplyRetryCount = retryDecision.attempt;
            lastApplyRetryAt = retryQueuedNow;
            nextState = "merged";
            workflowRunUrl = null;
            message = retryDecision.message;
          } catch (error) {
            nextApplyRetryCount = retryDecision.attempt;
            lastApplyRetryAt = retryQueuedNow;
            nextState = "merged";
            workflowRunUrl = null;
            message = `${retryDecision.message} Dispatch failed: ${
              error instanceof Error ? error.message : "unknown error"
            }`;
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
    updated_at: nowIso(),
    apply_retry_count: nextApplyRetryCount,
    last_apply_retry_at: lastApplyRetryAt,
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
    created_at: nowIso(),
    updated_at: nowIso(),
    apply_retry_count: 0,
    last_apply_retry_at: null,
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
      `- Managed by: \`bird-lg-autopeer\``,
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
    challenge.methods = methodsFromMaintainers(maintainers, []);

    if (challenge.methods.length === 0) {
      const message = configuredOidcProviders(env).length > 0
        ? `AS${asn} does not expose supported registry SSH or PGP auth methods. Use one of the configured OIDC login options instead.`
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
