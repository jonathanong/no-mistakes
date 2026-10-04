-- Permanent graph nodes must propagate retirement through database-conditioned views.
CREATE VIEW public.middle AS SELECT * FROM public.accounts;
CREATE TEMP VIEW "Audit.Database".pg_temp.orders AS SELECT * FROM public.middle;
CREATE VIEW order_lines AS SELECT * FROM "Audit.Database".pg_temp.orders;
SELECT * FROM orders;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;
SELECT * FROM order_lines;
