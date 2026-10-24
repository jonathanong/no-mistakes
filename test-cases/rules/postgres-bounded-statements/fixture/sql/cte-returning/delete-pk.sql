WITH deleted AS (
  DELETE FROM orders WHERE orders.id = $1 RETURNING account_id AS batch_id
)
UPDATE accounts SET name = 'deleted' FROM deleted WHERE accounts.id = deleted.batch_id;
