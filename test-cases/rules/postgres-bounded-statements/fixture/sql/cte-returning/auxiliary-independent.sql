WITH updated AS (
  UPDATE orders SET status = 'processed' FROM accounts a
  WHERE orders.id = $1 AND a.name = 'other' RETURNING orders.account_id AS batch_id
)
UPDATE accounts SET name = 'updated' FROM updated WHERE accounts.id = updated.batch_id;
