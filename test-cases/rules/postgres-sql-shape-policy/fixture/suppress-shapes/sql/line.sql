SELECT id FROM accounts WHERE id NOT IN (SELECT account_id FROM bans); -- no-mistakes-disable-line postgres-sql-shape-policy
