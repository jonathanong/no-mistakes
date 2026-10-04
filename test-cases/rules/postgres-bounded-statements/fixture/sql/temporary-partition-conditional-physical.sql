-- Physical parent and child precede the temporary partition tree on this path.
CREATE SCHEMA real_schema;
CREATE TABLE real_schema.accounts(id integer) PARTITION BY RANGE (id);
CREATE TABLE real_schema.orders PARTITION OF real_schema.accounts FOR VALUES FROM (0) TO (10);
CREATE TABLE public.orders(id integer PRIMARY KEY);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
SET search_path = real_schema, pg_temp, public;
ALTER TABLE accounts DETACH PARTITION orders;
DROP TABLE pg_temp.accounts CASCADE;
RESET search_path;
SELECT * FROM orders;

-- Unqualified ATTACH reaches the physical tree; the standalone temp child survives.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders(id integer);
SET search_path = real_schema, pg_temp, public;
ALTER TABLE accounts ATTACH PARTITION orders FOR VALUES FROM (0) TO (10);
DROP TABLE pg_temp.accounts CASCADE;
RESET search_path;
SELECT * FROM orders;
