UPDATE accounts SET active = false
WHERE id NOT IN (SELECT account_id FROM bans);
DELETE FROM accounts
WHERE (SELECT COUNT(*) FROM orders) = 0;
