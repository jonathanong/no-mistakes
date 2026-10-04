-- The bare source can resolve in missing_schema, but explicit pg_temp is definite.
CREATE TEMP TABLE accounts(id uuid);
SET search_path = missing_schema, pg_temp, public;
CREATE VIEW orders AS SELECT a.id FROM accounts AS a CROSS JOIN pg_temp.accounts AS t;
RESET search_path;
SELECT * FROM orders WHERE id = '00000000-0000-0000-0000-000000000001';
-- A predicate on the opaque view cannot pin the catalog accounts join target.
SELECT * FROM orders AS o JOIN public.accounts AS a ON a.id = o.id WHERE o.id = '00000000-0000-0000-0000-000000000001';
