-- Missing inventory cannot prove whether DROP reached the temporary relation.
-- Keep the joined target unbounded even if the selected catalog has unique keys.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = unknown_schema, pg_temp, public;
DROP TABLE accounts;
RESET search_path;
SELECT 1 FROM accounts a JOIN orders o ON o.id = a.id WHERE a.id = $1;

-- The implicit current-user schema cannot be proven empty by a partial catalog.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = "$user", pg_temp, public;
DROP TABLE accounts;
RESET search_path;
SELECT 1 FROM accounts a JOIN orders o ON o.id = a.id WHERE a.id = $1;
