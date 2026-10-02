CREATE TABLE public.orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
CREATE TEMP TABLE orders (id int, computed int);
ALTER TABLE orders ADD COLUMN extra int;
DROP TABLE orders;
