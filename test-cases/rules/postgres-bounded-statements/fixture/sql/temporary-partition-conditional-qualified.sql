-- Explicit pg_temp names select the temporary tree despite physical namesakes.
CREATE SCHEMA real_schema;
CREATE TABLE real_schema.accounts(id integer) PARTITION BY RANGE (id);
CREATE TABLE real_schema.orders PARTITION OF real_schema.accounts FOR VALUES FROM (0) TO (10);
CREATE TABLE public.orders(id integer PRIMARY KEY);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
BEGIN;
SET LOCAL search_path = real_schema, pg_temp, public;
ALTER TABLE pg_temp.accounts DETACH PARTITION pg_temp.orders;
ROLLBACK;
DROP TABLE pg_temp.accounts CASCADE;
RESET search_path;
SELECT * FROM orders;

-- Outside the rolled-back transaction, qualified DETACH succeeds and preserves orders.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
SET search_path = real_schema, pg_temp, public;
ALTER TABLE pg_temp.accounts DETACH PARTITION pg_temp.orders;
DROP TABLE pg_temp.accounts CASCADE;
RESET search_path;
SELECT * FROM orders;

-- Qualified ATTACH owns that surviving temp child despite the earlier namesake.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
SET search_path = real_schema, pg_temp, public;
ALTER TABLE pg_temp.accounts ATTACH PARTITION pg_temp.orders FOR VALUES FROM (0) TO (10);
DROP TABLE pg_temp.accounts CASCADE;
RESET search_path;
SELECT * FROM orders;
