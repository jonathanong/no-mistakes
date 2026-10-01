SELECT e.id FROM events e JOIN accounts a ON a.id = e.owner_id WHERE account_id = $1;
