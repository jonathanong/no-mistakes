SELECT orders.id FROM orders JOIN orders p ON p.account_id = $1 AND p.id = orders.parent_id WHERE orders.account_id = $1;
