SELECT o.id FROM orders o JOIN accounts a ON o.account_id = $1;
