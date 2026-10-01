SELECT e.id FROM accounts a JOIN events e ON e.account_id = a.id WHERE a.id = $1;
