WITH updated AS (
  WITH shadows AS (SELECT id FROM orders WHERE orders.status = 'queued')
  UPDATE orders SET status = 'processed'
  WHERE orders.id IN (SELECT id FROM shadows WHERE shadows.id = $1)
  RETURNING account_id AS batch_id
)
UPDATE accounts SET name = 'updated' FROM updated WHERE accounts.id = updated.batch_id;
