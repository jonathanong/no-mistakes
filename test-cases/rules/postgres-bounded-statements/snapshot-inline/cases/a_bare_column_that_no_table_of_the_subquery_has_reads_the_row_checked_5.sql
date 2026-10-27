UPDATE accounts SET name = $1 WHERE id IN ($2, (SELECT id FROM currencies LIMIT 1));
