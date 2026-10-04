CREATE TABLE "Audit.Database".pg_temp.accounts (id int);
SET search_path = empty_schema, pg_temp;
SELECT * FROM accounts;
SET search_path = public, pg_temp;
SELECT * FROM accounts;
RESET search_path;
SELECT * FROM accounts;
DROP TABLE pg_temp.accounts;
SELECT * FROM accounts;
