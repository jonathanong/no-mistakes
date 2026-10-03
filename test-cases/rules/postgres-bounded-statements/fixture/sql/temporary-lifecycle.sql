-- Transaction rollback and savepoints restore the permanent name after temp creation.
BEGIN;
CREATE TEMP TABLE accounts(id uuid);
ROLLBACK;
SELECT * FROM accounts;
BEGIN;
SAVEPOINT before_temp;
CREATE TEMP TABLE accounts(id uuid);
ROLLBACK TO before_temp;
SELECT * FROM accounts;
COMMIT;
-- Renaming a temp table frees the old permanent name.
CREATE TEMP TABLE accounts(id uuid);
ALTER TABLE accounts RENAME TO scratch;
SELECT * FROM accounts;
SELECT * FROM scratch;
DROP TABLE scratch;
-- PostgreSQL infers temporary views from their prepared temporary dependencies.
CREATE TEMP TABLE helper(id uuid);
CREATE VIEW accounts AS SELECT * FROM helper;
SELECT * FROM accounts;
CREATE VIEW orders AS SELECT * FROM accounts;
DROP TABLE helper CASCADE;
SELECT * FROM accounts;
SELECT * FROM orders;
-- Explicit pg_temp order and quoted qualifiers determine relation identity.
CREATE TEMP TABLE accounts(id uuid);
SET search_path=public,pg_temp;
SELECT * FROM accounts;
SELECT * FROM pg_temp.accounts;
RESET search_path;
SELECT * FROM accounts;
DROP TABLE "pg_temp".accounts;
SELECT * FROM accounts;
-- Savepoint release and transaction chaining retain exactly the committed identities.
BEGIN;
SAVEPOINT keep;
CREATE TEMP TABLE accounts(id uuid);
SAVEPOINT later;
CREATE TEMP TABLE orders(id uuid);
ROLLBACK TO keep;
SELECT * FROM accounts;
SELECT * FROM orders;
RELEASE SAVEPOINT keep;
COMMIT AND CHAIN;
CREATE TEMP TABLE accounts(id uuid);
ROLLBACK AND CHAIN;
SELECT * FROM accounts;
ROLLBACK;
-- LOCAL search-path changes expire at commit while committed temp tables remain.
BEGIN;
CREATE TEMP TABLE accounts(id uuid);
SET LOCAL search_path TO 'public, pg_temp';
SELECT * FROM accounts;
COMMIT;
SELECT * FROM accounts;
RESET ALL;
DROP TABLE accounts;
-- Renaming a temp dependency also renames the dependency recorded by a temp view.
CREATE TEMP TABLE helper(id uuid);
CREATE VIEW accounts AS SELECT * FROM helper;
ALTER TABLE helper RENAME TO helper_new;
DROP TABLE helper_new CASCADE;
SELECT * FROM accounts;
-- A source fragment may mention transaction identities established elsewhere.
RELEASE SAVEPOINT outside_source;
ROLLBACK TO outside_source;
ROLLBACK;
DROP TABLE IF EXISTS public.missing_orders;
ALTER TABLE public.orders RENAME TO archived_orders;
ALTER TABLE public.archived_orders ADD COLUMN extra integer;
SELECT * FROM accounts;
-- Numeric GUC values name a schema; omitted pg_temp still searches temporary relations first.
CREATE TEMP TABLE accounts(id uuid);
SET search_path=123;
SELECT * FROM accounts;
RESET search_path;
DROP TABLE accounts;
SELECT * FROM accounts;
-- A later SESSION assignment supersedes the LOCAL value restored at commit.
BEGIN;
CREATE TEMP TABLE accounts(id uuid);
SET LOCAL search_path=public,pg_temp;
SET SESSION search_path=public,pg_temp;
COMMIT;
SELECT * FROM accounts;
RESET search_path;
DROP TABLE accounts;
-- RESET also supersedes a LOCAL value instead of reviving the previous session path.
SET search_path=public,pg_temp;
BEGIN;
CREATE TEMP TABLE accounts(id uuid);
SET LOCAL search_path=pg_temp,public;
RESET ALL;
COMMIT;
SELECT * FROM accounts;
DROP TABLE accounts;
SELECT * FROM accounts;
-- DISCARD TEMP/ALL drops temp identities, while unrelated DISCARD modes retain them.
CREATE TEMP TABLE accounts(id uuid);
DISCARD PLANS;
DISCARD SEQUENCES;
SELECT * FROM accounts;
DISCARD TEMP;
SELECT * FROM accounts;
CREATE TEMP TABLE accounts(id uuid);
DISCARD ALL;
SELECT * FROM accounts;
-- Repeated LOCAL assignments retain the original session path for commit restoration.
BEGIN;
CREATE TEMP TABLE accounts(id uuid);
SET LOCAL search_path=public,pg_temp;
SET LOCAL search_path=pg_temp,public;
COMMIT;
SELECT * FROM accounts;
DROP TABLE accounts;
SELECT * FROM accounts;
