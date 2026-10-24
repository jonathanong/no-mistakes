-- The first write reads physical orders, not the later same-named CTE.
WITH earlier AS (
  UPDATE accounts SET name = 'changed'
  WHERE id IN (SELECT account_id FROM orders)
  RETURNING id
), orders AS (
  DELETE FROM orders WHERE id = $1 RETURNING account_id
)
SELECT 1;
