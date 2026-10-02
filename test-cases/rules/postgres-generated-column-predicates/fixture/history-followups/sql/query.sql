SELECT id FROM orders WHERE created_at > $1 OR alt_at > $2;
