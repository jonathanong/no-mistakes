SELECT 1 FROM accounts a JOIN profiles p ON p.account_id = a.id JOIN orders o USING (id) WHERE a.id = $1;
