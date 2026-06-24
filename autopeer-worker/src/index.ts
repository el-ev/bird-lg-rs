import {
  claimNodeOperationLock,
  deleteOperation,
  getAuthSession,
  getOperation,
  insertOperation,
  listActiveOperations,
  listOperationsForAsn,
  putOperation,
  releaseNodeOperationLock,
} from "./db";
import { branchName, GitHubClient } from "./github";
import type { GitHubWorkflowJob, GitHubWorkflowRun } from "./github";
import { resolveLocale, t } from "./i18n";
import {
  buildNodeViews,
  listSessionsForAsn,
  loadInventoryHosts,
  mutatePeerFile,
  validateSessionSpec,
} from "./network";
import {
  CreateSessionSchema,
  UpdateSessionSchema,
} from "./schemas";
import type {
  OperationFailureDetails,
  OperationKind,
  OperationRecord,
  OperationState,
  OperationStatus,
  PeerSessionSpec,
  SessionRecord,
  UiMessage,
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
  parseBody,
  parseConfiguredAsns,
  I18nError,
  isExpired,
  stripOperatorHints,
  readOptionalSecret,
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
const WORKFLOW_QUEUED_STALL_MS = 8 * 60 * 1000;
const WORKFLOW_IN_PROGRESS_STALL_MS = 20 * 60 * 1000;
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

function runtimeConfigResponse(request: Request, env: Env) {
  const origin = new URL(request.url).origin;
  return {
    autopeer_api_url: configuredUrl(env.AUTOPEER_API_URL) ?? origin,
    autopeer_site_url: configuredUrl(env.AUTOPEER_SITE_URL) ?? origin,
    looking_glass_url: configuredUrl(env.LOOKING_GLASS_URL),
    auth_url: env.AUTH_WORKER_URL,
    oidc_methods: [],
  };
}

function sessionCanImpersonate(env: Env, session: SessionRecord): boolean {
  return parseConfiguredAsns(env.HOST_ASNS).has(session.asn);
}

function sessionCanMutate(env: Env, session: SessionRecord): boolean {
  return !sessionCanImpersonate(env, session) || session.auth_method.kind === "host_impersonation";
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

type WorkflowGateConfig = {
  graceMs: number;
  pendingState: OperationState;
  successState: OperationState;
  shouldAttemptMerge: boolean;
  notStartedKey: string;
  waitStartKey: string;
  failedKey: string;
};

const CHECK_GATE_CONFIG: WorkflowGateConfig = {
  graceMs: CHECK_WORKFLOW_GRACE_MS,
  pendingState: "pending_checks",
  successState: "applying",
  shouldAttemptMerge: false,
  notStartedKey: "operation.message.check_not_started",
  waitStartKey: "operation.message.check_wait_start",
  failedKey: "operation.message.check_failed",
};

const APPLY_GATE_CONFIG: WorkflowGateConfig = {
  graceMs: CHECK_WORKFLOW_GRACE_MS + APPLY_WORKFLOW_GRACE_MS,
  pendingState: "applying",
  successState: "pending_merge",
  shouldAttemptMerge: true,
  notStartedKey: "operation.message.apply_not_started",
  waitStartKey: "operation.message.apply_wait_start",
  failedKey: "operation.message.apply_failed",
};

function decideWorkflowGate(
  operation: Pick<OperationRecord, "created_at">,
  run: ValidationWorkflowRun | undefined,
  config: WorkflowGateConfig,
  now = Date.now(),
): PreMergeGateDecision {
  if (!run) {
    const createdAt = Date.parse(operation.created_at);
    if (Number.isFinite(createdAt) && now - createdAt > config.graceMs) {
      return { state: "failed", message: uiMessage(config.notStartedKey), shouldAttemptMerge: false };
    }
    return { state: config.pendingState, message: uiMessage(config.waitStartKey), shouldAttemptMerge: false };
  }

  if (run.status !== "completed") {
    return { state: config.pendingState, message: buildOperationMessage(config.pendingState), shouldAttemptMerge: false };
  }

  if (!["success", "neutral", "skipped"].includes(run.conclusion ?? "")) {
    return {
      state: "failed",
      message: uiMessage(config.failedKey, { conclusion: run.conclusion ?? "unknown" }),
      shouldAttemptMerge: false,
    };
  }

  return { state: config.successState, message: buildOperationMessage(config.successState), shouldAttemptMerge: config.shouldAttemptMerge };
}

export function decideCheckGate(
  operation: Pick<OperationRecord, "created_at">,
  validationRun: ValidationWorkflowRun | undefined,
  now = Date.now(),
): PreMergeGateDecision {
  return decideWorkflowGate(operation, validationRun, CHECK_GATE_CONFIG, now);
}

export function decideApplyGate(
  operation: Pick<OperationRecord, "created_at">,
  applyRun: ValidationWorkflowRun | undefined,
  now = Date.now(),
): PreMergeGateDecision {
  return decideWorkflowGate(operation, applyRun, APPLY_GATE_CONFIG, now);
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

function isWorkflowRunStalled(
  run: GitHubWorkflowRun,
  now = Date.now(),
): boolean {
  const createdAt = Date.parse(run.created_at);
  if (!Number.isFinite(createdAt)) return false;
  const age = now - createdAt;
  if (run.status === "queued") return age > WORKFLOW_QUEUED_STALL_MS;
  if (run.status === "in_progress") return age > WORKFLOW_IN_PROGRESS_STALL_MS;
  return false;
}

async function maybeNotifyStalledRun(
  env: Env,
  github: GitHubClient,
  operation: OperationRecord,
  run: GitHubWorkflowRun | undefined,
  stage: "check" | "apply",
  alreadyNotifiedAt: string | null,
): Promise<string | null> {
  if (alreadyNotifiedAt || !operation.pr_number || !run) return alreadyNotifiedAt;
  if (!isWorkflowRunStalled(run)) return alreadyNotifiedAt;
  const mention = env.STALL_NOTIFY_MENTION?.trim();
  if (!mention) return alreadyNotifiedAt;

  const ageMin = Math.max(1, Math.round((Date.now() - Date.parse(run.created_at)) / 60000));
  const body =
    `${mention} the ${stage} workflow ` +
    `([run #${run.id}](${run.html_url})) on this PR has been \`${run.status}\` ` +
    `for ~${ageMin} min — the self-hosted runner may be stuck.`;
  try {
    await github.createIssueComment(operation.pr_number, body);
    return nowIso();
  } catch (error) {
    console.error(
      `stall notify: failed to comment on PR ${operation.pr_number} for op ${operation.id}`,
      error,
    );
    return alreadyNotifiedAt;
  }
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
  let stalledNotifiedAt: string | null = operation.stalled_notified_at ?? null;

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
    stalledNotifiedAt = await maybeNotifyStalledRun(
      env,
      github,
      operation,
      validationRun,
      "check",
      stalledNotifiedAt,
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
      stalledNotifiedAt = await maybeNotifyStalledRun(
        env,
        github,
        operation,
        applyRun,
        "apply",
        stalledNotifiedAt,
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
    stalled_notified_at: stalledNotifiedAt,
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
  let sessionPayload: PeerSessionSpec | undefined;
  if (kind === "create") {
    const body = await parseBody(request, CreateSessionSchema);
    nodeName = body.node;
    sessionPayload = body.session;
  } else if (kind === "update") {
    const body = await parseBody(request, UpdateSessionSchema);
    sessionPayload = body.session;
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
