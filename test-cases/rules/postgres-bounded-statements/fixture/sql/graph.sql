-- Bounded: accounts is joined by its primary key from one order.
SELECT o.id FROM orders o JOIN accounts a ON a.id = o.account_id WHERE o.id = $1;
-- Bounded: the target rows come from a subquery with a LIMIT, by primary key or by ctid.
DELETE FROM sessions WHERE id IN (SELECT id FROM sessions ORDER BY id LIMIT $1);
UPDATE orders SET status = 'stale' WHERE ctid IN (SELECT ctid FROM orders ORDER BY id LIMIT $1);
-- Unbounded: one account has many orders.
SELECT o.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = $1;
-- Unbounded: the target is selected through a non-key column.
DELETE FROM orders USING accounts a WHERE orders.account_id = a.id AND a.email = $1;
-- Not judged: a relation the catalog does not describe.
SELECT 1 FROM audit_log;
