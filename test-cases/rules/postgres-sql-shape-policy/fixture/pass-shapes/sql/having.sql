SELECT account_id FROM orders GROUP BY account_id HAVING COUNT(*) > 0;
