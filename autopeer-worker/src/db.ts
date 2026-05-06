import type {
  AuthMethod,
  ChallengeRecord,
  RegistryEmailAuthRequestRecord,
  OidcAuthRequestRecord,
  OperationKind,
  OperationRecord,
  OperationState,
  PeerSessionSpec,
  SessionRecord,
} from "./types";
import { HttpError, nowIso, uiMessage } from "./utils";

type ConsumeChallengeResult =
  | { kind: "available"; challenge: ChallengeRecord }
  | { kind: "missing" | "expired" | "consumed" };

function mapChallengeRow(row: Record<string, unknown>): ChallengeRecord {
  return {
    id: String(row.id),
    asn: String(row.asn),
    challenge_text: String(row.challenge_text),
    methods: JSON.parse(String(row.method_snapshot)) as AuthMethod[],
    maintainers: JSON.parse(String(row.maintainer_snapshot)),
    created_at: String(row.created_at),
    expires_at: String(row.expires_at),
  };
}

function mapSessionRow(row: Record<string, unknown>): SessionRecord {
  return {
    token: String(row.token),
    asn: String(row.asn),
    effective_mnt: String(row.effective_mnt),
    auth_method: {
      ...(JSON.parse(String(row.auth_method)) as AuthMethod),
      provider:
        typeof row.auth_provider === "string" && row.auth_provider.length > 0
          ? String(row.auth_provider)
          : (JSON.parse(String(row.auth_method)) as AuthMethod).provider,
    },
    created_at: String(row.created_at),
    expires_at: String(row.expires_at),
  };
}

function mapOperationRow(row: Record<string, unknown>): OperationRecord {
  return {
    id: String(row.id),
    asn: String(row.asn),
    node: String(row.node),
    kind: String(row.kind) as OperationKind,
    state: String(row.state) as OperationState,
    branch: String(row.branch),
    pr_number: row.pr_number === null ? null : Number(row.pr_number),
    pr_node_id: row.pr_node_id === null ? null : String(row.pr_node_id),
    pull_request_url: row.pull_request_url === null ? null : String(row.pull_request_url),
    workflow_run_url: row.workflow_run_url === null ? null : String(row.workflow_run_url),
    message:
      row.message === null || row.message === undefined
        ? null
        : (JSON.parse(String(row.message)) as OperationRecord["message"]),
    failure_details:
      row.failure_details === null || row.failure_details === undefined
        ? null
        : (JSON.parse(String(row.failure_details)) as OperationRecord["failure_details"]),
    created_at: String(row.created_at),
    updated_at: String(row.updated_at),
    session_snapshot:
      row.session_snapshot === null
        ? null
        : (JSON.parse(String(row.session_snapshot)) as PeerSessionSpec),
  };
}

function mapOidcAuthRequestRow(row: Record<string, unknown>): OidcAuthRequestRecord {
  return {
    state: String(row.state),
    challenge_id: String(row.challenge_id),
    provider: String(row.provider),
    nonce: String(row.nonce),
    code_verifier: String(row.code_verifier),
    redirect_uri: String(row.redirect_uri),
    session_token: row.session_token === null ? null : String(row.session_token),
    created_at: String(row.created_at),
    expires_at: String(row.expires_at),
  };
}

function mapRegistryEmailAuthRequestRow(
  row: Record<string, unknown>,
): RegistryEmailAuthRequestRecord {
  return {
    challenge_id: String(row.challenge_id),
    effective_mnt: String(row.effective_mnt),
    email_snapshot: JSON.parse(String(row.email_snapshot)) as string[],
    code: String(row.code),
    token: String(row.token),
    session_token: row.session_token === null ? null : String(row.session_token),
    locale: row.locale === null || row.locale === undefined ? null : String(row.locale),
    created_at: String(row.created_at),
    expires_at: String(row.expires_at),
  };
}

function mapNodeOperationLockRow(
  row: Record<string, unknown>,
): { node: string; operation_id: string; created_at: string; updated_at: string } {
  return {
    node: String(row.node),
    operation_id: String(row.operation_id),
    created_at: String(row.created_at),
    updated_at: String(row.updated_at),
  };
}

export async function putChallenge(env: Env, record: ChallengeRecord): Promise<void> {
  await env.DB.prepare(
    `INSERT OR REPLACE INTO auth_challenges
      (id, asn, challenge_text, maintainer_snapshot, method_snapshot, created_at, expires_at)
      VALUES (?, ?, ?, ?, ?, ?, ?)`,
  )
    .bind(
      record.id,
      record.asn,
      record.challenge_text,
      JSON.stringify(record.maintainers),
      JSON.stringify(record.methods),
      record.created_at,
      record.expires_at,
    )
    .run();
}

export async function getChallenge(env: Env, id: string): Promise<ChallengeRecord | null> {
  const row = await env.DB.prepare(
    `SELECT id, asn, challenge_text, maintainer_snapshot, method_snapshot, created_at, expires_at
      FROM auth_challenges WHERE id = ? AND consumed_at IS NULL`,
  )
    .bind(id)
    .first<Record<string, unknown>>();

  return row ? mapChallengeRow(row) : null;
}

export async function consumeFreshChallenge(
  env: Env,
  id: string,
): Promise<ConsumeChallengeResult> {
  const now = nowIso();
  const claimed = await env.DB.prepare(
    `UPDATE auth_challenges
      SET consumed_at = ?
      WHERE id = ? AND consumed_at IS NULL AND expires_at > ?
      RETURNING id, asn, challenge_text, maintainer_snapshot, method_snapshot, created_at, expires_at`,
  )
    .bind(now, id, now)
    .first<Record<string, unknown>>();

  if (claimed) {
    return {
      kind: "available",
      challenge: mapChallengeRow(claimed),
    };
  }

  const row = await env.DB.prepare(
    `SELECT id, expires_at, consumed_at FROM auth_challenges WHERE id = ?`,
  )
    .bind(id)
    .first<Record<string, unknown>>();

  if (!row) {
    return { kind: "missing" };
  }
  if (typeof row.consumed_at === "string" && row.consumed_at.length > 0) {
    return { kind: "consumed" };
  }
  if (Date.parse(String(row.expires_at)) <= Date.now()) {
    return { kind: "expired" };
  }

  return { kind: "missing" };
}

export async function deleteChallenge(env: Env, id: string): Promise<void> {
  await env.DB.prepare("DELETE FROM auth_challenges WHERE id = ?").bind(id).run();
}

export async function putOidcAuthRequest(env: Env, record: OidcAuthRequestRecord): Promise<void> {
  await env.DB.prepare(
    `INSERT OR REPLACE INTO oidc_auth_requests
      (state, challenge_id, provider, nonce, code_verifier, redirect_uri, session_token, created_at, expires_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
  )
    .bind(
      record.state,
      record.challenge_id,
      record.provider,
      record.nonce,
      record.code_verifier,
      record.redirect_uri,
      record.session_token ?? null,
      record.created_at,
      record.expires_at,
    )
    .run();
}

export async function getOidcAuthRequest(env: Env, state: string): Promise<OidcAuthRequestRecord | null> {
  const row = await env.DB.prepare(
    `SELECT state, challenge_id, provider, nonce, code_verifier, redirect_uri, session_token, created_at, expires_at
      FROM oidc_auth_requests WHERE state = ?`,
  )
    .bind(state)
    .first<Record<string, unknown>>();

  return row ? mapOidcAuthRequestRow(row) : null;
}

export async function deleteOidcAuthRequest(env: Env, state: string): Promise<void> {
  await env.DB.prepare("DELETE FROM oidc_auth_requests WHERE state = ?").bind(state).run();
}

export async function putRegistryEmailAuthRequest(
  env: Env,
  record: RegistryEmailAuthRequestRecord,
): Promise<void> {
  await env.DB.prepare(
    `INSERT OR REPLACE INTO registry_email_auth_requests
      (challenge_id, effective_mnt, email_snapshot, code, token, session_token, locale, created_at, expires_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
  )
    .bind(
      record.challenge_id,
      record.effective_mnt,
      JSON.stringify(record.email_snapshot),
      record.code,
      record.token,
      record.session_token ?? null,
      record.locale ?? null,
      record.created_at,
      record.expires_at,
    )
    .run();
}

export async function getRegistryEmailAuthRequest(
  env: Env,
  challengeId: string,
): Promise<RegistryEmailAuthRequestRecord | null> {
  const row = await env.DB.prepare(
    `SELECT challenge_id, effective_mnt, email_snapshot, code, token, session_token, locale, created_at, expires_at
      FROM registry_email_auth_requests WHERE challenge_id = ?`,
  )
    .bind(challengeId)
    .first<Record<string, unknown>>();

  return row ? mapRegistryEmailAuthRequestRow(row) : null;
}

export async function getRegistryEmailAuthRequestByToken(
  env: Env,
  token: string,
): Promise<RegistryEmailAuthRequestRecord | null> {
  const row = await env.DB.prepare(
    `SELECT challenge_id, effective_mnt, email_snapshot, code, token, session_token, locale, created_at, expires_at
      FROM registry_email_auth_requests WHERE token = ?`,
  )
    .bind(token)
    .first<Record<string, unknown>>();

  return row ? mapRegistryEmailAuthRequestRow(row) : null;
}

export async function consumeCompletedRegistryEmailAuthRequestByToken(
  env: Env,
  token: string,
): Promise<RegistryEmailAuthRequestRecord | null> {
  const row = await env.DB.prepare(
    `DELETE FROM registry_email_auth_requests
      WHERE token = ? AND session_token IS NOT NULL
      RETURNING challenge_id, effective_mnt, email_snapshot, code, token, session_token, locale, created_at, expires_at`,
  )
    .bind(token)
    .first<Record<string, unknown>>();

  return row ? mapRegistryEmailAuthRequestRow(row) : null;
}

export async function deleteRegistryEmailAuthRequest(
  env: Env,
  challengeId: string,
): Promise<void> {
  await env.DB.prepare(
    "DELETE FROM registry_email_auth_requests WHERE challenge_id = ?",
  )
    .bind(challengeId)
    .run();
}

export async function putAuthSession(env: Env, record: SessionRecord): Promise<void> {
  await env.DB.prepare(
    `INSERT OR REPLACE INTO auth_sessions
      (token, asn, effective_mnt, auth_method, auth_provider, created_at, expires_at)
      VALUES (?, ?, ?, ?, ?, ?, ?)`,
  )
    .bind(
      record.token,
      record.asn,
      record.effective_mnt,
      JSON.stringify(record.auth_method),
      record.auth_method.provider ?? null,
      record.created_at,
      record.expires_at,
    )
    .run();
}

export async function getAuthSession(env: Env, token: string): Promise<SessionRecord | null> {
  const row = await env.DB.prepare(
    `SELECT token, asn, effective_mnt, auth_method, auth_provider, created_at, expires_at
      FROM auth_sessions WHERE token = ?`,
  )
    .bind(token)
    .first<Record<string, unknown>>();

  return row ? mapSessionRow(row) : null;
}

function operationBindValues(record: OperationRecord): unknown[] {
  return [
    record.id,
    record.asn,
    record.node,
    record.kind,
    record.state,
    record.branch,
    record.session_snapshot ? JSON.stringify(record.session_snapshot) : null,
    record.pr_number ?? null,
    record.pr_node_id ?? null,
    record.pull_request_url ?? null,
    record.workflow_run_url ?? null,
    record.message ? JSON.stringify(record.message) : null,
    record.failure_details ? JSON.stringify(record.failure_details) : null,
    record.created_at,
    record.updated_at,
  ];
}

const OPERATION_COLUMNS = `(id, asn, node, kind, state, branch, session_snapshot, pr_number, pr_node_id, pull_request_url, workflow_run_url, message, failure_details, created_at, updated_at)`;
const OPERATION_PLACEHOLDERS = `(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`;

export async function putOperation(env: Env, record: OperationRecord): Promise<void> {
  await env.DB.prepare(
    `INSERT OR REPLACE INTO operations ${OPERATION_COLUMNS} VALUES ${OPERATION_PLACEHOLDERS}`,
  )
    .bind(...operationBindValues(record))
    .run();
}

export async function insertOperation(env: Env, record: OperationRecord): Promise<boolean> {
  try {
    await env.DB.prepare(
      `INSERT INTO operations ${OPERATION_COLUMNS} VALUES ${OPERATION_PLACEHOLDERS}`,
    )
      .bind(...operationBindValues(record))
      .run();
    return true;
  } catch {
    return false;
  }
}

export async function deleteOperation(env: Env, id: string): Promise<void> {
  await env.DB.prepare(`DELETE FROM operations WHERE id = ?`).bind(id).run();
}

export async function getOperation(env: Env, id: string): Promise<OperationRecord | null> {
  const row = await env.DB.prepare(
    `SELECT id, asn, node, kind, state, branch, session_snapshot, pr_number, pr_node_id, pull_request_url, workflow_run_url, message, failure_details, created_at, updated_at
      FROM operations WHERE id = ?`,
  )
    .bind(id)
    .first<Record<string, unknown>>();

  return row ? mapOperationRow(row) : null;
}

export async function listOperationsForAsn(env: Env, asn: string): Promise<OperationRecord[]> {
  const result = await env.DB.prepare(
    `SELECT id, asn, node, kind, state, branch, session_snapshot, pr_number, pr_node_id, pull_request_url, workflow_run_url, message, failure_details, created_at, updated_at
      FROM operations WHERE asn = ? ORDER BY updated_at DESC`,
  )
    .bind(asn)
    .all<Record<string, unknown>>();

  return result.results.map(mapOperationRow);
}

export async function listActiveOperations(env: Env): Promise<OperationRecord[]> {
  const result = await env.DB.prepare(
    `SELECT id, asn, node, kind, state, branch, session_snapshot, pr_number, pr_node_id, pull_request_url, workflow_run_url, message, failure_details, created_at, updated_at
      FROM operations WHERE state NOT IN ('completed', 'failed', 'conflict') AND pr_number IS NOT NULL ORDER BY created_at ASC`,
  )
    .all<Record<string, unknown>>();

  return result.results.map(mapOperationRow);
}

export async function claimNodeOperationLock(
  env: Env,
  node: string,
  operationId: string,
): Promise<string> {
  const now = nowIso();
  await env.DB.prepare(
    `INSERT OR IGNORE INTO node_operation_locks
      (node, operation_id, created_at, updated_at)
      VALUES (?, ?, ?, ?)`,
  )
    .bind(node, operationId, now, now)
    .run();

  const row = await env.DB.prepare(
    `SELECT node, operation_id, created_at, updated_at
      FROM node_operation_locks WHERE node = ?`,
  )
    .bind(node)
    .first<Record<string, unknown>>();

  if (!row) {
    throw new HttpError(uiMessage("error.node.lock.unreadable", { node }), 500);
  }

  const lock = mapNodeOperationLockRow(row);
  if (lock.operation_id === operationId) {
    await env.DB.prepare(
      `UPDATE node_operation_locks
        SET updated_at = ?
        WHERE node = ? AND operation_id = ?`,
    )
      .bind(now, node, operationId)
      .run();
  }

  return lock.operation_id;
}

export async function releaseNodeOperationLock(
  env: Env,
  node: string,
  operationId: string,
): Promise<void> {
  await env.DB.prepare(
    `DELETE FROM node_operation_locks WHERE node = ? AND operation_id = ?`,
  )
    .bind(node, operationId)
    .run();
}
