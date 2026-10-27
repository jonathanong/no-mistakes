SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders;
