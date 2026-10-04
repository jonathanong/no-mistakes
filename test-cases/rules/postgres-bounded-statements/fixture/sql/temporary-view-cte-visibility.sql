-- A nonrecursive CTE does not hide the earlier temporary table inside its own body.
CREATE TEMP TABLE helper(id integer);
CREATE VIEW accounts AS WITH helper AS (SELECT id FROM helper LIMIT 1)
  SELECT id FROM helper;
SELECT * FROM accounts;

-- RECURSIVE also exposes later CTE aliases inside an earlier body.
CREATE VIEW orders AS WITH RECURSIVE first AS (SELECT * FROM helper),
  helper AS (SELECT NULL::integer AS id) SELECT * FROM first;
SELECT * FROM orders;
DROP TABLE helper CASCADE;
SELECT * FROM accounts;
SELECT * FROM orders;

-- A later CTE alias does not hide a physical relation in the first CTE body.
CREATE TEMP TABLE future(id integer);
CREATE VIEW accounts AS WITH first AS (SELECT id FROM future LIMIT 1),
  future AS (SELECT 1 AS id) SELECT id FROM first;
SELECT * FROM accounts;
DROP TABLE future CASCADE;
SELECT * FROM accounts;

-- Recursive self-reference belongs to the CTE, not the unrelated temp table.
CREATE TEMP TABLE helper(id integer);
CREATE VIEW accounts AS WITH RECURSIVE helper(id) AS (
  SELECT 1 UNION ALL SELECT id + 1 FROM helper WHERE id < 2
) SELECT id FROM helper;
SELECT * FROM accounts;
