SELECT id FROM events WHERE account_id = ANY($1::uuid[]) AND kind = 'login';
