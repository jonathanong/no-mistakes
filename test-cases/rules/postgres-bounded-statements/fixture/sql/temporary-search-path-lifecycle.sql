-- A DROP behind unproved earlier schemas may remove the temporary relation. Retire its
-- identity so a later catalog read cannot be hidden, including after RESET search_path.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = missing_schema, pg_temp, public;
DROP TABLE accounts;
SELECT * FROM accounts;
RESET search_path;
SELECT * FROM accounts;

-- A possibly temporary rename must not leave the old name definitely temporary.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = missing_schema, pg_temp, public;
ALTER TABLE accounts RENAME TO archived_accounts;
SELECT * FROM accounts;
RESET search_path;
SELECT * FROM accounts;

-- CASCADE also retires a dependent temporary view, exposing its catalog namesake.
CREATE TEMP TABLE accounts(id uuid);
CREATE TEMP VIEW orders AS SELECT * FROM accounts;
SET search_path = missing_schema, pg_temp, public;
DROP TABLE accounts CASCADE;
RESET search_path;
SELECT * FROM orders;
