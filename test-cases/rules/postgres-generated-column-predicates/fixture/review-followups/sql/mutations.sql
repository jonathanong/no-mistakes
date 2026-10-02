UPDATE orders SET id = $1
WHERE created_at > $2;
DELETE FROM orders
WHERE created_at > $1;
WITH changed AS (DELETE FROM orders WHERE created_at > $1 RETURNING id) SELECT * FROM changed;
