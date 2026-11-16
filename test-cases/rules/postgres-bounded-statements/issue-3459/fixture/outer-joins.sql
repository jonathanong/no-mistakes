SELECT 1 FROM profiles p RIGHT JOIN accounts a ON a.id = p.account_id AND p.account_id = $1;
SELECT 1 FROM profiles p RIGHT JOIN accounts a ON p.account_id = a.id WHERE p.account_id = $1;
SELECT 1 FROM accounts a FULL JOIN profiles p ON a.id = p.account_id AND p.account_id = $1;
SELECT 1 FROM accounts a FULL JOIN profiles p ON p.account_id = a.id WHERE p.account_id = $1;
