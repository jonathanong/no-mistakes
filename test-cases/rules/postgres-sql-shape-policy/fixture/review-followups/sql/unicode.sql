SELECT id FROM accounts
WHERE (ñ NOT IN (SELECT a FROM bans))
AND (SELECT COUNT(*) FROM orders) != 0;
