DELETE FROM accounts WHERE id = ANY (SELECT id FROM currencies LIMIT 1);
