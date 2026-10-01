SELECT id FROM events WHERE (account_id = $1 AND kind = 'a') OR (account_id = $2 AND kind = 'b');
