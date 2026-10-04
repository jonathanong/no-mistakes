-- A materialized view stays physical; only its temporary dependents are tracked.
CREATE MATERIALIZED VIEW public.source AS SELECT * FROM public.accounts;
CREATE TEMP VIEW orders AS SELECT * FROM public.source;
CREATE VIEW order_lines AS SELECT * FROM orders;
SELECT * FROM orders;
DROP MATERIALIZED VIEW other.source CASCADE;
SELECT * FROM orders;
DROP MATERIALIZED VIEW public.source RESTRICT;
SELECT * FROM orders;
DROP MATERIALIZED VIEW public.source CASCADE;
SELECT * FROM orders;
SELECT * FROM order_lines;
-- A temporary namesake does not replace the qualified materialized view identity.
CREATE TEMP TABLE source(id uuid);
CREATE TEMP VIEW orders AS SELECT * FROM public.source;
DROP TABLE source CASCADE;
SELECT * FROM orders;
DROP MATERIALIZED VIEW public.source CASCADE;
SELECT * FROM orders;
-- Rollback restores the temporary view and its transitive dependent.
CREATE TEMP VIEW orders AS SELECT * FROM public.source;
CREATE VIEW order_lines AS SELECT * FROM orders;
BEGIN;
SAVEPOINT before_drop;
DROP MATERIALIZED VIEW public.source CASCADE;
SELECT * FROM order_lines;
ROLLBACK TO before_drop;
SELECT * FROM order_lines;
DROP MATERIALIZED VIEW public.source CASCADE;
SELECT * FROM orders;
ROLLBACK;
SELECT * FROM orders;
-- Creating a materialized view must not acquire a temporary identity.
DROP VIEW orders CASCADE;
CREATE MATERIALIZED VIEW orders AS SELECT * FROM public.accounts;
SELECT * FROM orders;
