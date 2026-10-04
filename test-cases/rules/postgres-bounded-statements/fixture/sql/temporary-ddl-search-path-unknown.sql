-- DEFAULT is a role/server setting, not a literal schema membership proof.
SET search_path = DEFAULT;
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
SET search_path = 123;
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
-- Unsupported expression forms never become schema-membership evidence.
SET search_path = -1;
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
SET search_path = '';
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
-- Simple string lists and directly quoted identifiers have known membership.
SET search_path = 'public, pg_temp';
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
SET search_path = "default", pg_temp;
CREATE TEMP VIEW orders AS SELECT * FROM other.accounts;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
