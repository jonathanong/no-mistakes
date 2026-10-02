UPDATE accounts SET status = 'x' RETURNING (SELECT id FROM orders);
DELETE FROM accounts RETURNING ARRAY[(SELECT id FROM orders)];
