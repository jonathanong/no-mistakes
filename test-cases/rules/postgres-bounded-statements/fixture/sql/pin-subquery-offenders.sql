-- An uncapped SELECT reads its unbounded pin subquery even when another predicate pins the outer key.
SELECT 1 FROM accounts WHERE id = $1 AND id IN (SELECT account_id FROM orders);
SELECT 1 FROM accounts WHERE id IN (SELECT account_id FROM orders);
-- DML reports target rows only; its key pin still bounds the target.
UPDATE accounts SET name = 'x' WHERE id = $1 AND id IN (SELECT account_id FROM orders);
DELETE FROM accounts WHERE id = $1 AND id IN (SELECT account_id FROM orders);
