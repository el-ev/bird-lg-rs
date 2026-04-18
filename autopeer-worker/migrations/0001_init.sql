CREATE TABLE IF NOT EXISTS auth_challenges (
  id TEXT PRIMARY KEY,
  asn TEXT NOT NULL,
  challenge_text TEXT NOT NULL,
  maintainer_snapshot TEXT NOT NULL,
  method_snapshot TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS auth_sessions (
  token TEXT PRIMARY KEY,
  asn TEXT NOT NULL,
  effective_mnt TEXT NOT NULL,
  auth_method TEXT NOT NULL,
  auth_provider TEXT,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS operations (
  id TEXT PRIMARY KEY,
  asn TEXT NOT NULL,
  node TEXT NOT NULL,
  kind TEXT NOT NULL,
  state TEXT NOT NULL,
  branch TEXT NOT NULL,
  session_snapshot TEXT,
  pr_number INTEGER,
  pr_node_id TEXT,
  pull_request_url TEXT,
  workflow_run_url TEXT,
  message TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_auth_challenges_asn
  ON auth_challenges (asn);

CREATE INDEX IF NOT EXISTS idx_auth_sessions_asn
  ON auth_sessions (asn);

CREATE INDEX IF NOT EXISTS idx_operations_asn_node
  ON operations (asn, node);
