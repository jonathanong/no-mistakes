DELETE FROM accounts WHERE id = (SELECT id FROM currencies LIMIT 1);
