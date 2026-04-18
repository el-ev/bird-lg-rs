CREATE TABLE IF NOT EXISTS oidc_auth_requests (
  state TEXT PRIMARY KEY,
  challenge_id TEXT NOT NULL,
  provider TEXT NOT NULL,
  nonce TEXT NOT NULL,
  code_verifier TEXT NOT NULL,
  redirect_uri TEXT NOT NULL,
  session_token TEXT,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_oidc_auth_requests_challenge
  ON oidc_auth_requests (challenge_id);
