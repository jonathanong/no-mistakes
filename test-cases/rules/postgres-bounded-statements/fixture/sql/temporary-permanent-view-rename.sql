-- A qualified namesake rename cannot move a permanent view's true source edge.
CREATE VIEW public.middle AS SELECT * FROM public.accounts;
CREATE TEMP VIEW orders AS SELECT * FROM public.middle;
ALTER TABLE other.accounts RENAME TO archived_accounts;
DROP TABLE public.accounts RESTRICT;
SELECT * FROM orders;
ALTER TABLE public.accounts RENAME TO archived_accounts;
DROP TABLE public.archived_accounts CASCADE;
SELECT * FROM orders;
