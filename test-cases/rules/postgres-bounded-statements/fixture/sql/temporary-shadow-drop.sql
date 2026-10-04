-- real_schema.accounts shadows the temporary table before DROP. Once the physical
-- shadow is gone, RESET exposes the same temporary rows and their unknown key shape.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = real_schema, pg_temp, public;
DROP TABLE accounts;
SELECT 1 FROM accounts a JOIN orders o ON o.id = a.id WHERE a.id = $1;
RESET search_path;
SELECT 1 FROM accounts a JOIN orders o ON o.id = a.id WHERE a.id = $1;
DROP TABLE pg_temp.accounts;

-- Complete empty-schema evidence instead proves that DROP reached pg_temp.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = empty_schema, pg_temp, public;
DROP TABLE accounts;
RESET search_path;
SELECT 1 FROM accounts a JOIN orders o ON o.id = a.id WHERE a.id = $1;
DROP TABLE public.orders;
