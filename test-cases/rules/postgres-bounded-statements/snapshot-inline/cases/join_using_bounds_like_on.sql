SELECT 1 FROM accounts a LEFT JOIN orders o USING (id) WHERE o.status = $1;
