SELECT id::text, uuid_extract_timestamp(id) FROM accounts ORDER BY id::text;
SELECT 1 FROM accounts WHERE id = $1::uuid;
SELECT 1 FROM accounts WHERE id >= CAST($1 AS uuid);
SELECT 1 FROM accounts WHERE lower(email) = $1;
SELECT 1 FROM accounts WHERE custom::text = $1 OR array::text = $1;
SELECT 1 FROM text_ids WHERE id::text = $1;
SELECT 1 FROM accounts a WHERE text((SELECT t.id FROM text_ids t)) = $1;
-- The inner predicate is bare; the outer function must not own the subquery's column.
SELECT 1 FROM accounts a WHERE text((SELECT t.id FROM text_ids t WHERE t.id = $1)) = $2;
