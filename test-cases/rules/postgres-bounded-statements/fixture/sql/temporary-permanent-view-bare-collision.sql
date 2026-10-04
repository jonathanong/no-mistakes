-- A rejected bare rename must not donate an edge to the existing destination.
CREATE VIEW public.middle AS SELECT * FROM public.accounts;
CREATE VIEW public.taken AS SELECT * FROM public.order_lines;
CREATE TEMP VIEW orders AS SELECT * FROM public.middle;
ALTER TABLE middle RENAME TO taken;
DROP VIEW public.taken CASCADE;
SELECT * FROM orders;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;

-- A bare rename without a collision still moves its qualified node.
CREATE VIEW public.free AS SELECT * FROM public.order_lines;
ALTER TABLE free RENAME TO moved;
CREATE TEMP VIEW orders AS SELECT * FROM public.moved;
SELECT * FROM orders;
DROP TABLE public.order_lines CASCADE;
SELECT * FROM orders;

-- A collision in a later schema must not reject a valid earlier-schema rename.
CREATE TABLE public.accounts (id integer);
CREATE TABLE public.order_lines (id integer);
CREATE VIEW public.middle AS SELECT * FROM public.accounts;
CREATE VIEW public.taken AS SELECT * FROM public.order_lines;
CREATE VIEW other.middle AS SELECT * FROM other.accounts;
SET search_path = other, public, pg_temp;
ALTER TABLE middle RENAME TO taken;
CREATE TEMP VIEW orders AS SELECT * FROM other.taken;
DROP VIEW public.taken CASCADE;
SELECT * FROM pg_temp.orders;
DROP TABLE other.accounts CASCADE;
RESET search_path;
SELECT * FROM orders;
