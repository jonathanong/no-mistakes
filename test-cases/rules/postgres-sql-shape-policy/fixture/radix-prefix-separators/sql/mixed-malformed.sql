SELECT * FROM orders LIMIT 0o_17;
BROKEN MIGRATION;
UPDATE orders SET amount = 0o_17 WHERE id = $1;
INSERT INTO orders (id, amount) VALUES ($1, 0b_10);
SELECT * FROM orders LIMIT 0b_10;
