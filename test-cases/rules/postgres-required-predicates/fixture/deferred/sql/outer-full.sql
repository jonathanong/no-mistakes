SELECT e.id FROM accounts a FULL JOIN orders e ON e.account_id = $1;
