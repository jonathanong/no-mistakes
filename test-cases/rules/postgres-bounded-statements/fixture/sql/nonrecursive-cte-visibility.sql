-- The CTE's definition reads the base table, not its own renamed projection.
DELETE FROM accounts WHERE id IN (WITH accounts AS (SELECT id AS candidate FROM accounts WHERE id = $1) SELECT candidate FROM accounts LIMIT 1);
DELETE FROM accounts WHERE id IN (WITH accounts(candidate) AS (SELECT id FROM accounts WHERE id = $1) SELECT candidate FROM accounts LIMIT 1);
-- Earlier non-recursive CTEs are visible to later definitions.
DELETE FROM accounts WHERE id IN (WITH accounts(candidate) AS (SELECT id FROM accounts WHERE id = $1), next AS (SELECT candidate FROM accounts) SELECT candidate FROM next LIMIT 1);
-- Recursive names remain visible while their own definitions are traversed.
DELETE FROM accounts WHERE id IN (WITH RECURSIVE walk(candidate) AS (SELECT id FROM accounts WHERE id = $1 UNION ALL SELECT candidate FROM walk WHERE false) SELECT candidate FROM walk LIMIT 1);
-- An actual outer reference must not acquire a finite proof.
DELETE FROM accounts a WHERE id IN (WITH accounts AS (SELECT a.id AS candidate FROM accounts b WHERE b.id = $1) SELECT candidate FROM accounts LIMIT 1);
