CREATE MATERIALIZED VIEW public.orders AS SELECT * FROM public.accounts;
CREATE TEMP TABLE orders(id int);
-- This wrong-kind DROP fails; continuing the script must preserve the temporary table.
DROP MATERIALIZED VIEW orders CASCADE;
SELECT * FROM orders;
CREATE TEMP VIEW order_lines AS SELECT * FROM public.orders;
DROP MATERIALIZED VIEW public.orders CASCADE;
SELECT * FROM orders;
SELECT * FROM order_lines;
DROP TABLE orders;
CREATE TEMP VIEW orders AS SELECT * FROM public.accounts;
DROP MATERIALIZED VIEW orders;
SELECT * FROM orders;
DROP MATERIALIZED VIEW pg_temp.orders CASCADE;
SELECT * FROM orders;
DROP VIEW orders;
SELECT * FROM orders;
