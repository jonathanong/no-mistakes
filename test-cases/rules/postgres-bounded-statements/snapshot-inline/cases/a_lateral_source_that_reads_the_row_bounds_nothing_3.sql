SELECT 1 FROM accounts a, LATERAL (SELECT o.id FROM orders o WHERE o.account_id = a.id) x WHERE a.id = $1;
