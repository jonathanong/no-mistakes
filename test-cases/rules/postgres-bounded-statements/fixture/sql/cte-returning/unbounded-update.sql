WITH updated AS (
  UPDATE orders SET status = 'processed' WHERE orders.status = 'queued' RETURNING account_id AS batch_id
)
UPDATE accounts SET name = 'updated' FROM updated WHERE accounts.id = updated.batch_id;
