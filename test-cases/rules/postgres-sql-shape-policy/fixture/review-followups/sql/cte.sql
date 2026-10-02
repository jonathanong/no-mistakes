WITH a AS (SELECT id FROM accounts WHERE id NOT IN (SELECT account_id FROM bans)),
b AS (SELECT id FROM accounts WHERE id NOT IN (SELECT account_id FROM bans))
SELECT * FROM a;
