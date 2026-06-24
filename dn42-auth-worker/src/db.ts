import type {
  AuthMethod,
  ChallengeRecord,
  RegistryEmailAuthRequestRecord,
  OidcAuthRequestRecord,
  SessionRecord,
} from "./types";
import { HttpError, nowIso, uiMessage } from "./utils";

export interface AuthDbEnv {
  DB: D1Database;
}

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
    site_return_url: row.site_return_url === null || row.site_return_url === undefined ? null : String(row.site_return_url),
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
    site_return_url: row.site_return_url === null || row.site_return_url === undefined ? null : String(row.site_return_url),
  };
}

export async function putChallenge(env: AuthDbEnv, record: ChallengeRecord): Promise<void> {
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

export async function getChallenge(env: AuthDbEnv, id: string): Promise<ChallengeRecord | null> {
  const row = await env.DB.prepare(
    `SELECT id, asn, challenge_text, maintainer_snapshot, method_snapshot, created_at, expires_at
      FROM auth_challenges WHERE id = ? AND consumed_at IS NULL`,
  )
    .bind(id)
    .first<Record<string, unknown>>();

  return row ? mapChallengeRow(row) : null;
}

export async function consumeFreshChallenge(
  env: AuthDbEnv,
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

export async function deleteChallenge(env: AuthDbEnv, id: string): Promise<void> {
  await env.DB.prepare("DELETE FROM auth_challenges WHERE id = ?").bind(id).run();
}

export async function putOidcAuthRequest(env: AuthDbEnv, record: OidcAuthRequestRecord): Promise<void> {
  await env.DB.prepare(
    `INSERT OR REPLACE INTO oidc_auth_requests
      (state, challenge_id, provider, nonce, code_verifier, redirect_uri, session_token, created_at, expires_at, site_return_url)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
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
      record.site_return_url ?? null,
    )
    .run();
}

export async function getOidcAuthRequest(env: AuthDbEnv, state: string): Promise<OidcAuthRequestRecord | null> {
  const row = await env.DB.prepare(
    `SELECT state, challenge_id, provider, nonce, code_verifier, redirect_uri, session_token, created_at, expires_at, site_return_url
      FROM oidc_auth_requests WHERE state = ?`,
  )
    .bind(state)
    .first<Record<string, unknown>>();

  return row ? mapOidcAuthRequestRow(row) : null;
}

export async function deleteOidcAuthRequest(env: AuthDbEnv, state: string): Promise<void> {
  await env.DB.prepare("DELETE FROM oidc_auth_requests WHERE state = ?").bind(state).run();
}

export async function putRegistryEmailAuthRequest(
  env: AuthDbEnv,
  record: RegistryEmailAuthRequestRecord,
): Promise<void> {
  await env.DB.prepare(
    `INSERT OR REPLACE INTO registry_email_auth_requests
      (challenge_id, effective_mnt, email_snapshot, code, token, session_token, locale, created_at, expires_at, site_return_url)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
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
      record.site_return_url ?? null,
    )
    .run();
}

export async function getRegistryEmailAuthRequest(
  env: AuthDbEnv,
  challengeId: string,
): Promise<RegistryEmailAuthRequestRecord | null> {
  const row = await env.DB.prepare(
    `SELECT challenge_id, effective_mnt, email_snapshot, code, token, session_token, locale, created_at, expires_at, site_return_url
      FROM registry_email_auth_requests WHERE challenge_id = ?`,
  )
    .bind(challengeId)
    .first<Record<string, unknown>>();

  return row ? mapRegistryEmailAuthRequestRow(row) : null;
}

export async function getRegistryEmailAuthRequestByToken(
  env: AuthDbEnv,
  token: string,
): Promise<RegistryEmailAuthRequestRecord | null> {
  const row = await env.DB.prepare(
    `SELECT challenge_id, effective_mnt, email_snapshot, code, token, session_token, locale, created_at, expires_at, site_return_url
      FROM registry_email_auth_requests WHERE token = ?`,
  )
    .bind(token)
    .first<Record<string, unknown>>();

  return row ? mapRegistryEmailAuthRequestRow(row) : null;
}

export async function consumeCompletedRegistryEmailAuthRequestByToken(
  env: AuthDbEnv,
  token: string,
): Promise<RegistryEmailAuthRequestRecord | null> {
  const row = await env.DB.prepare(
    `DELETE FROM registry_email_auth_requests
      WHERE token = ? AND session_token IS NOT NULL
      RETURNING challenge_id, effective_mnt, email_snapshot, code, token, session_token, locale, created_at, expires_at, site_return_url`,
  )
    .bind(token)
    .first<Record<string, unknown>>();

  return row ? mapRegistryEmailAuthRequestRow(row) : null;
}

export async function deleteRegistryEmailAuthRequest(
  env: AuthDbEnv,
  challengeId: string,
): Promise<void> {
  await env.DB.prepare(
    "DELETE FROM registry_email_auth_requests WHERE challenge_id = ?",
  )
    .bind(challengeId)
    .run();
}

export async function putAuthSession(env: AuthDbEnv, record: SessionRecord): Promise<void> {
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

export async function getAuthSession(env: AuthDbEnv, token: string): Promise<SessionRecord | null> {
  const row = await env.DB.prepare(
    `SELECT token, asn, effective_mnt, auth_method, auth_provider, created_at, expires_at
      FROM auth_sessions WHERE token = ?`,
  )
    .bind(token)
    .first<Record<string, unknown>>();

  return row ? mapSessionRow(row) : null;
}
