WITH picked AS (SELECT id FROM accounts WHERE id = $1) UPDATE accounts t SET name = 'x' FROM picked WHERE t.id = picked.id;
WITH picked AS (SELECT id FROM accounts WHERE id = ANY($1::uuid[])) DELETE FROM accounts t USING picked WHERE t.id = picked.id;
WITH picked AS (UPDATE accounts SET name = 'x' WHERE id = $1 RETURNING id) UPDATE profiles t SET bio = 'y' FROM picked WHERE t.account_id = picked.id;
WITH picked AS (SELECT id FROM accounts) UPDATE accounts t SET name = 'x' FROM picked WHERE t.id = picked.id;
WITH picked AS (SELECT id FROM accounts) DELETE FROM accounts t USING picked WHERE t.id = picked.id;
