-- no-mistakes-disable-file postgres-sql-shape-policy
SELECT id FROM accounts WHERE id NOT IN (SELECT account_id FROM bans);
