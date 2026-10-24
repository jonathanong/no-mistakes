WITH selected AS (
  SELECT account_id AS batch_id FROM orders WHERE orders.id = $1
)
UPDATE accounts SET name = 'selected' FROM selected WHERE accounts.id = selected.batch_id;
