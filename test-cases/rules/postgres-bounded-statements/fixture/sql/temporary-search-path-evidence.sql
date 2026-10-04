CREATE TEMP TABLE accounts(id uuid);

-- The snapshot explicitly proves this schema does not exist.
SET search_path = missing_schema, pg_temp, public;
SELECT * FROM accounts;
SELECT * FROM "accounts";
-- Resolve a conditional temp source through both derived and pin subqueries.
SELECT * FROM (SELECT * FROM accounts) a;
SELECT * FROM orders o WHERE o.id IN (SELECT id FROM accounts);
-- A temporary source cannot use the permanent catalog's unique key to bound a join.
SELECT 1 FROM accounts a JOIN orders o ON o.id = a.id WHERE a.id = $1;
RESET search_path;
SELECT 1 FROM accounts a JOIN orders o ON o.id = a.id WHERE a.id = $1;

-- This existing schema has a complete, empty relation-name inventory.
SET search_path = empty_schema, pg_temp, public;
SELECT * FROM accounts;

-- An earlier real relation wins, but is outside the selected public catalog.
SET search_path = real_schema, pg_temp, public;
SELECT * FROM accounts;

-- A missing evidence entry is unknown, so the public catalog stays conservative.
SET search_path = unknown_schema, pg_temp, public;
SELECT * FROM accounts;

-- Explicit qualification bypasses the unqualified search-path decision.
SELECT * FROM pg_temp.accounts;
SELECT * FROM public.accounts;
SELECT * FROM audit.accounts;

-- The selected catalog's own relation precedes pg_temp here.
SET search_path = public, pg_temp;
SELECT * FROM accounts;

-- SET LOCAL restores the previous unknown path and its conservative result at COMMIT.
SET search_path = unknown_schema, pg_temp, public;
BEGIN;
SET LOCAL search_path = missing_schema, pg_temp, public;
SELECT * FROM accounts;
COMMIT;
SELECT * FROM accounts;
