SELECT 1 FROM accounts a JOIN orders o ON o.account_id = a.id JOIN order_lines l ON l.order_id = o.id WHERE a.id = $1;
