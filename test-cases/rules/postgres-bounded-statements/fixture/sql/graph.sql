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
-- Unbounded: an unknown relation can supply every account id, so it bounds nothing.
UPDATE accounts a SET name = 'x' FROM audit_log l WHERE a.id = l.account_id;
-- Unbounded: IS NOT DISTINCT FROM matches NULL, which a nullable unique column repeats.
DELETE FROM contacts WHERE email IS NOT DISTINCT FROM $1;
-- Bounded: the primary key cannot be NULL.
DELETE FROM contacts WHERE id IS NOT DISTINCT FROM $1;
-- Unbounded: WITH TIES returns every row tied with the last one.
SELECT id FROM orders ORDER BY account_id FETCH FIRST 10 ROWS WITH TIES;
-- Bounded: ONLY is a hard cap, and count is the built-in aggregate.
SELECT id FROM orders ORDER BY id FETCH FIRST 10 ROWS ONLY;
SELECT pg_catalog.count(*) FROM orders;
-- Unbounded: a function named count in another schema is called once per row.
SELECT app.count(id) FROM orders;
-- Bounded: a pinned order bounds its account, and the account its profile.
SELECT 1 FROM orders o JOIN accounts a ON a.id = o.account_id JOIN profiles p ON p.account_id = a.id WHERE o.id = $1;
