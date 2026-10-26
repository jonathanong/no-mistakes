SELECT 1 FROM users, sessions WHERE id::text = $1;
