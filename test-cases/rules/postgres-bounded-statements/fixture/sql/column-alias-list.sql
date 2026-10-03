-- Positional aliases can rename a non-key column to a catalog key name.
SELECT 1 FROM accounts AS a(id, real_id) WHERE a.id = $1;
SELECT 1 FROM accounts AS a WHERE a.id = $1;
SELECT 1 FROM accounts AS a(id, real_id) JOIN orders o ON a.id = o.account_id WHERE o.id = $1;
SELECT 1 FROM accounts AS a(id, real_id) JOIN orders o USING (id) WHERE o.id = $1;
