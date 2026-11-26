WITH target_orders AS (
  SELECT id FROM orders WHERE status = 'pending'
)
UPDATE orders o
SET
  cached_name = (SELECT a1.display_name FROM accounts a1 WHERE a1.id = o.account_id),
  cached_tier = (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id)
FROM target_orders t
WHERE o.id = t.id;

WITH removed AS (
  DELETE FROM orders o
  WHERE (SELECT a1.suspended FROM accounts a1 WHERE a1.id = o.account_id)
    AND (SELECT a2.archived FROM accounts a2 WHERE a2.id = o.account_id)
  RETURNING o.id
)
SELECT id FROM removed;

MERGE INTO orders o
USING incoming_orders s
ON o.id = s.id
WHEN MATCHED THEN
  UPDATE SET
    cached_name = (SELECT a1.display_name FROM accounts a1 WHERE a1.id = s.account_id),
    cached_tier = (SELECT a2.tier FROM accounts a2 WHERE a2.id = s.account_id);
