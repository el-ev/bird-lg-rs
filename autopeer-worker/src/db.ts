import type {
  OperationKind,
  OperationRecord,
  OperationState,
  PeerSessionSpec,
} from "./types";
import { HttpError, nowIso, uiMessage } from "./utils";

export {
  getAuthSession,
} from "dn42-auth-worker/db";

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
    stalled_notified_at:
      row.stalled_notified_at === null || row.stalled_notified_at === undefined
        ? null
        : String(row.stalled_notified_at),
    created_at: String(row.created_at),
    updated_at: String(row.updated_at),
    session_snapshot:
      row.session_snapshot === null
        ? null
        : (JSON.parse(String(row.session_snapshot)) as PeerSessionSpec),
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
    record.stalled_notified_at ?? null,
    record.created_at,
    record.updated_at,
  ];
}

const OPERATION_COLUMNS = `(id, asn, node, kind, state, branch, session_snapshot, pr_number, pr_node_id, pull_request_url, workflow_run_url, message, failure_details, stalled_notified_at, created_at, updated_at)`;
const OPERATION_PLACEHOLDERS = `(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`;

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
    `SELECT id, asn, node, kind, state, branch, session_snapshot, pr_number, pr_node_id, pull_request_url, workflow_run_url, message, failure_details, stalled_notified_at, created_at, updated_at
      FROM operations WHERE id = ?`,
  )
    .bind(id)
    .first<Record<string, unknown>>();

  return row ? mapOperationRow(row) : null;
}

export async function listOperationsForAsn(env: Env, asn: string): Promise<OperationRecord[]> {
  const result = await env.DB.prepare(
    `SELECT id, asn, node, kind, state, branch, session_snapshot, pr_number, pr_node_id, pull_request_url, workflow_run_url, message, failure_details, stalled_notified_at, created_at, updated_at
      FROM operations WHERE asn = ? ORDER BY updated_at DESC`,
  )
    .bind(asn)
    .all<Record<string, unknown>>();

  return result.results.map(mapOperationRow);
}

export async function listActiveOperations(env: Env): Promise<OperationRecord[]> {
  const result = await env.DB.prepare(
    `SELECT id, asn, node, kind, state, branch, session_snapshot, pr_number, pr_node_id, pull_request_url, workflow_run_url, message, failure_details, stalled_notified_at, created_at, updated_at
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
