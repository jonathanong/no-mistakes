WITH updated AS (
  WITH shadows AS (
    SELECT account_id AS id, id AS source_id FROM orders WHERE orders.status = 'queued'
  )
  UPDATE orders SET status = 'processed'
  WHERE orders.id IN (SELECT source_id FROM shadows WHERE shadows.id = $1)
  RETURNING orders.id AS batch_id
)
UPDATE accounts SET name = 'updated' FROM updated WHERE accounts.id = updated.batch_id;
