ALTER TABLE auth_challenges
  ADD COLUMN consumed_at TEXT;

CREATE INDEX IF NOT EXISTS idx_auth_challenges_consumed_at
  ON auth_challenges (consumed_at);
