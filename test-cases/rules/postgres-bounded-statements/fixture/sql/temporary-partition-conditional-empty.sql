-- Complete empty-schema evidence lets the unqualified DETACH reach pg_temp.
CREATE SCHEMA empty_schema;
CREATE TABLE public.orders(id integer PRIMARY KEY);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
SET search_path = empty_schema, pg_temp, public;
ALTER TABLE accounts DETACH PARTITION orders;
DROP TABLE pg_temp.accounts CASCADE;
RESET search_path;
SELECT * FROM orders;
DROP TABLE pg_temp.orders;
SELECT * FROM orders;

-- The same proof lets an unqualified ATTACH give the temporary child an owner.
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders(id integer);
SET search_path = empty_schema, pg_temp, public;
ALTER TABLE accounts ATTACH PARTITION orders FOR VALUES FROM (0) TO (10);
DROP TABLE pg_temp.accounts CASCADE;
RESET search_path;
SELECT * FROM orders;
