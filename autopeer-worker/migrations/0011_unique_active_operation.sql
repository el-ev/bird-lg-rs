CREATE UNIQUE INDEX IF NOT EXISTS idx_operations_one_active_per_asn_node
  ON operations (asn, node)
  WHERE state NOT IN ('completed', 'failed', 'conflict');
