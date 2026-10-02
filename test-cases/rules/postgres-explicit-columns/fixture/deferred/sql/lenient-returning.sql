ALTER TABLE ignored ADD COLUMN n int GENERATED ALWAYS AS (1) VIRTUAL;
DELETE FROM orders
RETURNING
-- no-mistakes-disable-next-line postgres-explicit-columns
*;
UPDATE orders SET id = 1
RETURNING
-- no-mistakes-disable-next-line postgres-explicit-columns
orders.*;
INSERT INTO orders (id) VALUES (1)
RETURNING
-- no-mistakes-disable-next-line postgres-explicit-columns
*;
