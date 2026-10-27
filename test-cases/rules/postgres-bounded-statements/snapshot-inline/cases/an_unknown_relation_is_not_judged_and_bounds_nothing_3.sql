SELECT 1 FROM accounts a JOIN external e ON a.id = e.account_id WHERE e.id = $1;
