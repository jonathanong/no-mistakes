-- Both references follow the same temporary table through a definite rename.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = missing_schema, pg_temp, public;
CREATE VIEW orders AS SELECT a.id FROM accounts AS a CROSS JOIN pg_temp.accounts AS t;
ALTER TABLE pg_temp.accounts RENAME TO accounts_moved;
RESET search_path;
SELECT * FROM orders;
DROP TABLE accounts_moved CASCADE;
-- PostgreSQL now resolves the public namesake, not the dropped temporary view.
SELECT * FROM orders;
