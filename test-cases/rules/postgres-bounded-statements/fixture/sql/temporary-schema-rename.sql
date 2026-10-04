-- A schema rename changes qualified physical identities, not relation names.
CREATE TEMP VIEW orders AS SELECT * FROM audit.accounts;
CREATE VIEW order_lines AS SELECT * FROM orders;
ALTER SCHEMA audit RENAME TO archived;
DROP TABLE IF EXISTS audit.accounts CASCADE;
SELECT * FROM orders;
DROP TABLE archived.accounts CASCADE;
SELECT * FROM orders;
SELECT * FROM order_lines;
-- Renaming another schema leaves this source identity unchanged.
CREATE TEMP VIEW orders AS SELECT * FROM audit.accounts;
ALTER SCHEMA other RENAME TO other_archived;
DROP TABLE IF EXISTS other_archived.accounts CASCADE;
SELECT * FROM orders;
DROP TABLE audit.accounts CASCADE;
SELECT * FROM orders;
-- Quoted dots and case belong to one schema identifier.
CREATE TEMP VIEW orders AS SELECT * FROM "Audit.Schema"."Accounts";
ALTER SCHEMA "Audit.Schema" RENAME TO "Archived.Schema";
DROP TABLE IF EXISTS "archived.schema"."Accounts" CASCADE;
SELECT * FROM orders;
DROP TABLE "Archived.Schema"."Accounts" CASCADE;
SELECT * FROM orders;
-- Rollback restores both dependency identity and transitive temporary views.
CREATE TEMP VIEW orders AS SELECT * FROM audit.accounts;
CREATE VIEW order_lines AS SELECT * FROM orders;
BEGIN;
SAVEPOINT original_schema;
ALTER SCHEMA audit RENAME TO archived;
DROP TABLE archived.accounts CASCADE;
SELECT * FROM order_lines;
ROLLBACK TO original_schema;
DROP TABLE IF EXISTS archived.accounts CASCADE;
SELECT * FROM order_lines;
ALTER SCHEMA audit RENAME TO archived;
COMMIT;
DROP TABLE archived.accounts CASCADE;
SELECT * FROM orders;
-- A bare source remains ambiguous and matches schema-qualified drops conservatively.
CREATE TEMP VIEW orders AS SELECT * FROM accounts;
ALTER SCHEMA audit RENAME TO archived;
DROP TABLE archived.accounts CASCADE;
SELECT * FROM orders;
-- Database qualification is retained while the schema component changes.
CREATE TEMP VIEW orders AS SELECT * FROM "Db.Name".audit.accounts;
ALTER SCHEMA audit RENAME TO archived;
DROP TABLE IF EXISTS "Other".archived.accounts CASCADE;
SELECT * FROM orders;
DROP TABLE "Db.Name".archived.accounts CASCADE;
SELECT * FROM orders;
-- Unsupported qualified schema names and ownership changes must not rename sources.
CREATE TEMP VIEW orders AS SELECT * FROM audit.accounts;
ALTER SCHEMA catalog.audit RENAME TO archived;
ALTER SCHEMA audit RENAME TO catalog.archived;
ALTER SCHEMA audit OWNER TO CURRENT_USER;
SELECT * FROM orders;
DROP TABLE audit.accounts CASCADE;
SELECT * FROM orders;
