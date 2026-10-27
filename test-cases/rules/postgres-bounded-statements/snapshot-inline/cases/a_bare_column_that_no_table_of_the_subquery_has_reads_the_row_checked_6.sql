DELETE FROM accounts WHERE id IN (SELECT code FROM currencies WHERE id = $1 LIMIT 1);
