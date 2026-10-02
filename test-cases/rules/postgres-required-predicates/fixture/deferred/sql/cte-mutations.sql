WITH changed AS (UPDATE orders SET status = 'x' WHERE id = $1 RETURNING id) SELECT id FROM changed;
WITH changed AS (DELETE FROM orders WHERE id = $1 RETURNING id) SELECT id FROM changed;
WITH ids AS (SELECT 1) UPDATE orders SET status = 'x' WHERE id = $1;
WITH ids AS (SELECT 1) DELETE FROM orders WHERE id = $1;
WITH inserted AS (INSERT INTO logs (id) SELECT id FROM orders RETURNING id) SELECT id FROM inserted;
