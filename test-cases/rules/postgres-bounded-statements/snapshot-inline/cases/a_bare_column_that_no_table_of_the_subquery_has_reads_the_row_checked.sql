DELETE FROM accounts a WHERE a.id IN (SELECT id FROM currencies LIMIT 1);
