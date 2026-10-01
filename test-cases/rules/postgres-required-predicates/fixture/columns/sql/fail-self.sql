SELECT o.id FROM orders o JOIN orders p ON p.account_id = $1 WHERE o.id = p.id;
