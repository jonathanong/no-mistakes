SELECT 1 FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = $1;
