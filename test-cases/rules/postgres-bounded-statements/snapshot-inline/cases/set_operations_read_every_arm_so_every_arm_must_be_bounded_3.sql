SELECT id FROM accounts WHERE id = $1 INTERSECT SELECT account_id FROM orders;
