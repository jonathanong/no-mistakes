SELECT id FROM events WHERE account_id = $1 OR kind = 'login';
