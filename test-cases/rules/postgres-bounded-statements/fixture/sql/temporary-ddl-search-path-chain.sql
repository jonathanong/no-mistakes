SET search_path = public, pg_temp;
CREATE VIEW other.middle AS SELECT * FROM other.accounts;
CREATE TEMP VIEW orders AS SELECT * FROM other.middle;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
ALTER TABLE accounts RENAME TO archived_accounts;
DROP TABLE other.archived_accounts CASCADE;
SELECT * FROM orders;
DROP TABLE other.accounts CASCADE;
SELECT * FROM orders;
-- Cascaded graph nodes retain their creation identity after a path change.
SET search_path = other, pg_temp;
CREATE VIEW middle AS SELECT * FROM public.accounts;
CREATE TEMP VIEW orders AS SELECT * FROM other.middle;
SET search_path = public, pg_temp;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;
