SELECT 1 FROM accounts WHERE id::text = $1;
SELECT 1 FROM accounts WHERE CAST(id AS text) = $1;
SELECT 1 FROM accounts WHERE uuid_extract_timestamp(id) > $1;
SELECT 1 FROM orders o JOIN accounts a ON o.account_id::text = a.id::text;
SELECT id FROM accounts GROUP BY id HAVING id::text > $1;
UPDATE accounts SET email = $1 WHERE id::text = $2;
DELETE FROM orders WHERE uuid_extract_timestamp(id) < $1;
-- UUID columns need no key-name convention or key constraint.
SELECT 1 FROM accounts WHERE token::text = $1;
