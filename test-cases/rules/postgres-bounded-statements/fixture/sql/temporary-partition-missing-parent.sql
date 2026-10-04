-- A missing qualified parent cannot detach an attached temporary child.
CREATE TABLE orders(id integer);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
ALTER TABLE IF EXISTS public.accounts DETACH PARTITION pg_temp.orders;
DROP TABLE accounts;
SELECT * FROM orders;
