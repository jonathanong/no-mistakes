WITH selected AS (SELECT id FROM accounts WHERE id = $1)
SELECT * FROM selected FOR UPDATE;
SELECT * FROM accounts WHERE id = $1 FOR UPDATE;
