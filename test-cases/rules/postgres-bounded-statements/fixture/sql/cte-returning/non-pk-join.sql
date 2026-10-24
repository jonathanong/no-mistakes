WITH updated AS (
  UPDATE orders SET status = 'processed' WHERE orders.id = $1 RETURNING status AS next_status
)
UPDATE accounts SET name = 'updated' FROM updated WHERE accounts.name = updated.next_status;
