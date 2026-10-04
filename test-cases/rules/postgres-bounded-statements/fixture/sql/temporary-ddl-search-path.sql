-- Unknown paths retain conservative invalidation.
RESET search_path;
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
-- An explicitly excluded schema cannot own a bare DROP or RENAME target.
SET search_path = public, pg_temp;
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
SELECT * FROM orders;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
ALTER TABLE accounts RENAME TO archived_accounts;
DROP TABLE other.archived_accounts CASCADE;
SELECT * FROM orders;
DROP TABLE other.accounts CASCADE;
SELECT * FROM orders;
-- Exact qualified renames still move dependencies.
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
ALTER TABLE other.accounts RENAME TO archived_accounts;
DROP TABLE other.accounts CASCADE;
SELECT * FROM orders;
DROP TABLE other.archived_accounts CASCADE;
SELECT * FROM orders;
-- Membership does not prove which eligible physical schema owns a bare target.
SET search_path = public, pg_temp, other;
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
-- Quoted string syntax and role substitution remain unknown to this narrow proof.
SET search_path = '"other,public", pg_temp';
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
SET search_path = "$user", public, pg_temp;
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
-- A bare dependency has no schema identity to exclude.
SET search_path = public, pg_temp;
CREATE TEMP VIEW orders AS SELECT * FROM accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
