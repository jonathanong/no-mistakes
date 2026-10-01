SELECT EXISTS (SELECT * FROM orders WHERE account_id = $1);
