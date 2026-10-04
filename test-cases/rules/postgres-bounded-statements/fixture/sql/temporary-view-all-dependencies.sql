-- A scalar SELECT-list subquery can make a view implicitly temporary.
CREATE TEMP TABLE helper(id integer);
CREATE VIEW accounts AS SELECT (SELECT id FROM helper LIMIT 1) AS id;
SELECT * FROM accounts;
DROP TABLE helper CASCADE;
SELECT * FROM accounts;

-- A WHERE scalar subquery must also be retained for a cascading permanent drop.
CREATE TEMP VIEW orders AS SELECT id FROM public.accounts
  WHERE id = (SELECT id FROM public.order_lines LIMIT 1);
DROP TABLE public.order_lines CASCADE;
SELECT * FROM orders;

-- The CTE named helper shadows the unrelated temporary table of that name.
CREATE TEMP TABLE helper(id integer);
CREATE VIEW accounts AS WITH helper AS (SELECT 1 AS id)
  SELECT (SELECT id FROM helper LIMIT 1) AS id;
SELECT * FROM accounts;
