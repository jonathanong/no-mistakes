-- PostgreSQL ignores SET LOCAL outside a transaction, leaving public first.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = public, pg_temp;
SET LOCAL search_path = missing_schema, pg_temp, public;
SELECT * FROM accounts;

-- Inside BEGIN the same LOCAL path applies until COMMIT restores the session path.
BEGIN;
SET LOCAL search_path = missing_schema, pg_temp, public;
SELECT * FROM accounts;
COMMIT;
SELECT * FROM accounts;
