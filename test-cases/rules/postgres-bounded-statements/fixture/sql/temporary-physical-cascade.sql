-- Physical schema identity and temporary dependency identity remain distinct.
CREATE TEMP VIEW orders AS SELECT * FROM public.accounts;
CREATE VIEW order_lines AS SELECT * FROM orders;
SELECT * FROM orders;
DROP TABLE other.accounts CASCADE;
SELECT * FROM orders;
DROP TABLE public.accounts RESTRICT;
SELECT * FROM orders;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;
SELECT * FROM order_lines;
-- A view over only permanent tables must not acquire an inferred temporary identity.
CREATE VIEW orders AS SELECT * FROM public.accounts;
SELECT * FROM orders;
DROP VIEW orders;
-- Dropping a shadowing temporary table must not drop views over its physical namesake.
CREATE TEMP TABLE accounts(id uuid);
CREATE TEMP VIEW orders AS SELECT * FROM public.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;
-- Permanent renames preserve dependencies until the renamed base is dropped.
CREATE TEMP VIEW orders AS SELECT * FROM public.accounts;
ALTER TABLE public.accounts RENAME TO archived_accounts;
DROP TABLE public.archived_accounts CASCADE;
SELECT * FROM orders;
-- Bare physical reads may resolve to a schema-qualified drop target.
CREATE TEMP VIEW orders AS SELECT * FROM accounts;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;
-- An unrelated qualified rename must not erase a possible bare dependency.
CREATE TEMP VIEW orders AS SELECT * FROM accounts;
ALTER TABLE other.accounts RENAME TO archived_accounts;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;
-- A qualified namesake rename retains both physical and temporary dependency edges.
CREATE TEMP VIEW orders AS SELECT * FROM public.accounts;
CREATE VIEW order_lines AS SELECT * FROM orders;
ALTER TABLE other.accounts RENAME TO archived_accounts;
SELECT * FROM orders;
DROP TABLE public.accounts CASCADE;
SELECT * FROM order_lines;
-- Derived bodies and subquery pins retain their physical dependencies.
CREATE TEMP VIEW orders AS SELECT * FROM (SELECT * FROM public.accounts) a;
DROP TABLE public.accounts CASCADE;
SELECT * FROM orders;
CREATE TEMP VIEW orders AS SELECT * FROM public.accounts WHERE id IN (SELECT id FROM public.order_lines LIMIT 1);
DROP TABLE public.order_lines CASCADE;
SELECT * FROM orders;
