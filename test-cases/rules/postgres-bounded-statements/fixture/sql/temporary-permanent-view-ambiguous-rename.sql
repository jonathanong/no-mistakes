-- An unqualified source may still be public.accounts after an unrelated qualified rename.
CREATE VIEW public.middle AS SELECT * FROM accounts;
CREATE TEMP VIEW orders AS SELECT * FROM public.middle;
ALTER TABLE other.accounts RENAME TO archived_accounts;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;
