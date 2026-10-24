WITH changed AS (
  UPDATE orders SET status = 'processed' WHERE orders.id = $1
)
UPDATE accounts SET name = 'updated' WHERE accounts.id = $2;
