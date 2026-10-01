SELECT e.id FROM events e JOIN topics t ON t.id = e.id WHERE account_id = $1;
