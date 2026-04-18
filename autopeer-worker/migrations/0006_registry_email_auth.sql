CREATE TABLE IF NOT EXISTS registry_email_auth_requests (
  challenge_id TEXT PRIMARY KEY,
  effective_mnt TEXT NOT NULL,
  email_snapshot TEXT NOT NULL,
  code TEXT NOT NULL,
  token TEXT NOT NULL UNIQUE,
  session_token TEXT,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_registry_email_auth_requests_token
  ON registry_email_auth_requests (token);
