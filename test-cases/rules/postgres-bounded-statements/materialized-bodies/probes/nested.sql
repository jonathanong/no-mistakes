SELECT EXISTS (
  WITH nested_pool AS MATERIALIZED (SELECT id FROM edge_rows)
  SELECT 1 FROM nested_pool
);
