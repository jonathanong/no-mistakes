SELECT id FROM accounts
WHERE id NOT IN (SELECT account_id FROM bans);
SELECT id FROM accounts
WHERE id NOT IN (SELECT account_id FROM bans);
