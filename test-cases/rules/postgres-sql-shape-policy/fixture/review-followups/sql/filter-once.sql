SELECT COUNT(*) FILTER (WHERE account_id NOT IN (SELECT account_id FROM bans)) FROM orders;
SELECT COUNT(*) FILTER (WHERE (SELECT COUNT(*) FROM bans) > 0) FROM orders;
