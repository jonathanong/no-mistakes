-- no-mistakes-disable-next-line postgres-sql-shape-policy: reviewed
SELECT id FROM accounts WHERE id NOT IN (SELECT account_id FROM bans);
