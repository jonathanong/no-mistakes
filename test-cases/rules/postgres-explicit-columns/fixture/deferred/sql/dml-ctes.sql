WITH changed AS (UPDATE orders SET status = 'x' RETURNING *) SELECT id FROM changed;
WITH changed AS (DELETE FROM orders RETURNING *) SELECT id FROM changed;
WITH changed AS (INSERT INTO orders (id) VALUES (1) RETURNING *) SELECT id FROM changed;
WITH ids AS (SELECT 1) UPDATE orders SET status = 'x' RETURNING *;
WITH ids AS (SELECT 1) DELETE FROM orders RETURNING *;
