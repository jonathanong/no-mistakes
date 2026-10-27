SELECT 1 FROM accounts a LEFT JOIN profiles p ON p.account_id = a.id WHERE p.account_id = $1;
