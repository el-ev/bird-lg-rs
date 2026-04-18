CREATE TABLE IF NOT EXISTS node_operation_locks (
  node TEXT PRIMARY KEY,
  operation_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_node_operation_locks_operation_id
  ON node_operation_locks (operation_id);
