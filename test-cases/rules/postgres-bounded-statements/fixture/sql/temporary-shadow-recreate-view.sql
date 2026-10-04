-- Recreating a qualified permanent view restores its public shadow; the
-- temporary orders table must survive the subsequent unqualified DROP VIEW.
CREATE TEMP TABLE orders(id uuid);
SET search_path = public, pg_temp;
DROP TABLE orders;
CREATE VIEW public.orders AS SELECT id FROM public.accounts;
DROP VIEW orders;
RESET search_path;
SELECT 1 FROM orders o JOIN accounts a ON a.id = o.id WHERE a.id = $1;
