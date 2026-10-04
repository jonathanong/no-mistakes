-- PostgreSQL rejects a mixed-persistence ATTACH. Seeing a temporary child in
-- the conditional path must not make a physical parent own that child.
CREATE SCHEMA empty_schema;
CREATE TABLE public.accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders(id integer);
SET search_path = empty_schema, pg_temp, public;
ALTER TABLE public.accounts ATTACH PARTITION orders FOR VALUES FROM (0) TO (10);
SELECT * FROM orders;
