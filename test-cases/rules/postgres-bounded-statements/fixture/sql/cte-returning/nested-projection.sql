WITH updated AS (
  UPDATE orders SET status = 'processed' WHERE orders.id = $1 RETURNING account_id AS batch_id
)
SELECT 1 FROM accounts a JOIN (SELECT batch_id FROM updated) u ON a.id = u.batch_id;
