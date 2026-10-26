SELECT 1 FROM accounts WHERE (coalesce(id, $1))::text = $2;
SELECT 1 FROM accounts WHERE NOT (id::text IN ($1, $2));
SELECT 1 FROM accounts WHERE id::text BETWEEN $1 AND $2;
SELECT 1 FROM accounts WHERE id::text IS NULL;
SELECT 1 FROM accounts WHERE f(id => id) = $1;
UPDATE accounts a SET email = $1 FROM orders o WHERE a.id = o.account_id AND o.id::text = $2;
DELETE FROM orders o USING accounts a WHERE o.account_id = a.id AND a.id::text = $1;
