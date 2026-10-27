SELECT 1 FROM accounts a LEFT JOIN profiles p ON a.id = p.account_id AND p.account_id = $1;
