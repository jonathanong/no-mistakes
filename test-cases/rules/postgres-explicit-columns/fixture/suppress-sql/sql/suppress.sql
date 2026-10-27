SELECT id FROM orders;
SELECT
  -- no-mistakes-disable-next-line postgres-explicit-columns
  *
FROM orders;
DELETE FROM orders
-- no-mistakes-disable-next-line postgres-explicit-columns
RETURNING *;
