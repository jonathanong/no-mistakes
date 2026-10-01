SELECT id FROM accounts a WHERE (SELECT COUNT(*) FROM orders o WHERE o.account_id = a.id) > 0;
