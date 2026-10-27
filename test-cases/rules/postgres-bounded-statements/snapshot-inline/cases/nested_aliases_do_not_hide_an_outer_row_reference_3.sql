DELETE FROM accounts WHERE accounts.id IN (SELECT accounts.id FROM orders WHERE orders.id IN (SELECT id FROM accounts));
