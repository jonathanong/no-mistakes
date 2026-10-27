SELECT id FROM accounts WHERE id = $1 UNION ALL SELECT id FROM orders;
