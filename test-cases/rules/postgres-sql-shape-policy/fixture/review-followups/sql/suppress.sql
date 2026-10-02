SELECT id FROM accounts
-- no-mistakes-disable-next-line postgres-sql-shape-policy
WHERE id NOT /* comment */ IN (SELECT account_id FROM bans);
SELECT id FROM accounts
WHERE (SELECT COUNT(*) FROM orders) != 0; -- no-mistakes-disable-line postgres-sql-shape-policy
