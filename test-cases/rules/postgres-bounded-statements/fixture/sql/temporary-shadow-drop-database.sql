-- Only a snapshot for the named current database can prove this DROP reaches pg_temp.
CREATE TABLE "Audit.Database".pg_temp.accounts(id uuid);
SET search_path = empty_schema, pg_temp, public;
DROP TABLE accounts;
RESET search_path;
SELECT 1 FROM accounts a JOIN orders o ON o.id = a.id WHERE a.id = $1;
