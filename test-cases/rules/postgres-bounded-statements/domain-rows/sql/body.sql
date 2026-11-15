WITH pool AS MATERIALIZED (
  SELECT id FROM ranks WHERE owner_id = $1
)
SELECT id FROM pool LIMIT 1;

WITH flags_pool AS MATERIALIZED (
  SELECT id FROM flags WHERE owner_id = $1
)
SELECT id FROM flags_pool LIMIT 1;
