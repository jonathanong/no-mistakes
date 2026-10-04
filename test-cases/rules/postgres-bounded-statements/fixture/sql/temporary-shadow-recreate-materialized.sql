-- Materialized views use the same physical shadow lifetime as ordinary views.
CREATE TEMP TABLE orders(id uuid);
SET search_path = public, pg_temp;
DROP TABLE orders;
CREATE MATERIALIZED VIEW public.orders AS SELECT id FROM public.accounts;
DROP MATERIALIZED VIEW orders;
RESET search_path;
SELECT 1 FROM orders o JOIN accounts a ON a.id = o.id WHERE a.id = $1;
