WITH updated AS (
  WITH picked AS (SELECT id FROM orders WHERE orders.id = $1)
  UPDATE orders SET status = 'processed'
  WHERE orders.id IN (SELECT id FROM picked) RETURNING account_id AS batch_id
)
UPDATE accounts SET name = 'updated' FROM updated WHERE accounts.id = updated.batch_id;
