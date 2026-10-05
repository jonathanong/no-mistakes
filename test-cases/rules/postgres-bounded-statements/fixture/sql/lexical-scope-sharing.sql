-- Inner declarations shadow CTE bounds without changing the enclosing scope.
WITH scoped(id) AS (SELECT id FROM accounts WHERE id = $1)
SELECT * FROM (
  WITH scoped AS (SELECT id FROM orders WHERE id = $2)
  SELECT * FROM scoped
) AS inner_scope, scoped;

-- A recursive placeholder retains the visible explicit-column metadata.
WITH scoped(id) AS (SELECT id FROM accounts WHERE id = $1)
SELECT * FROM (
  WITH RECURSIVE scoped AS (
    SELECT id FROM orders WHERE id = $2
    UNION ALL
    SELECT id FROM scoped WHERE id = $3
  )
  SELECT * FROM scoped
) AS inner_scope, scoped;
