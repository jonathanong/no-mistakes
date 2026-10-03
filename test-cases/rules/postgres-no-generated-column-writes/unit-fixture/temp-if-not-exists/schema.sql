CREATE TABLE orders (id int, total int GENERATED ALWAYS AS (id) STORED);
-- Creates pg_temp.orders (shadowing the permanent table); it is not a no-op.
CREATE TEMP TABLE IF NOT EXISTS orders (id int, total int);
