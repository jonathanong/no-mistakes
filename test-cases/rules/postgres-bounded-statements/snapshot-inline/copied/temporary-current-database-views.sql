CREATE TEMP TABLE helper (id int);
-- The reference, not the unqualified destination, carries the database condition.
CREATE VIEW orders AS SELECT * FROM "Audit.Database".pg_temp.helper;
SELECT * FROM orders;
ALTER TABLE helper RENAME TO accounts;
SELECT * FROM orders;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
-- A pg_temp destination is temporary even with a permanent source.
CREATE VIEW "Audit.Database".pg_temp.orders AS SELECT * FROM public.accounts;
SELECT * FROM orders;
-- Renaming an unrelated temporary relation must keep the view's physical edge.
CREATE TABLE "Audit.Database".pg_temp.helper (id int);
ALTER TABLE "Audit.Database".pg_temp.helper RENAME TO temp_helper;
DROP TABLE "Audit.Database".pg_temp.temp_helper;
DROP VIEW "Audit.Database".pg_temp.orders;
SELECT * FROM orders;
BEGIN;
CREATE TABLE "Audit.Database".pg_temp.accounts (id int) ON COMMIT DROP;
SELECT * FROM accounts;
ALTER TABLE "Audit.Database".pg_temp.accounts RENAME TO order_lines;
COMMIT;
SELECT * FROM order_lines;
-- Unquoted database identifiers fold; quoted identifiers retain their exact case.
CREATE TABLE AuditDB.pg_temp.accounts (id int);
SELECT * FROM accounts;
DISCARD ALL;
SELECT * FROM accounts;
