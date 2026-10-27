SELECT id FROM orders;
SELECT
  -- postgres-explicit-columns
  *
FROM orders;
DELETE FROM orders
-- postgres-explicit-columns
RETURNING *;
