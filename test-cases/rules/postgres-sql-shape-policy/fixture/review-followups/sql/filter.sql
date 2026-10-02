SELECT COUNT(*) FILTER (
WHERE account_id NOT IN (SELECT account_id FROM bans)) FROM orders;
