DELETE FROM accounts a WHERE a.id IN (SELECT o.account_id FROM orders o WHERE o.account_id = a.id LIMIT 5);
