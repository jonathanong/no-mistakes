SELECT account_id FROM orders
GROUP BY account_id
HAVING account_id NOT IN (SELECT account_id FROM bans)
AND (SELECT COUNT(*) FROM bans) = 0;
