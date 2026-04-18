ALTER TABLE operations ADD COLUMN apply_retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE operations ADD COLUMN last_apply_retry_at TEXT;
