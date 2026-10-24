WITH updated AS (
  UPDATE orders SET status = 'processed' FROM accounts a
  WHERE orders.id = a.id RETURNING orders.account_id AS batch_id
)
UPDATE accounts SET name = 'updated' FROM updated WHERE accounts.id = updated.batch_id;
