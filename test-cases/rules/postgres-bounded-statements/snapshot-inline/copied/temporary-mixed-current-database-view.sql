-- The explicit current-database source remains conditional until catalog evaluation.
CREATE TEMP TABLE accounts(id integer);
SET search_path = missing_schema, pg_temp, public;
CREATE VIEW orders AS SELECT a.id FROM accounts AS a CROSS JOIN "Audit.Database".pg_temp.accounts AS t;
RESET search_path;
-- The view cannot lend a permanent orders key to the joined public accounts table.
SELECT * FROM orders AS o JOIN public.accounts AS a ON a.id = o.id WHERE o.id = 1;
