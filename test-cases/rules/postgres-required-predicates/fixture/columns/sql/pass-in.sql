SELECT id FROM accounts WHERE account_id IN (SELECT id FROM events WHERE account_id = $1);
