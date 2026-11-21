SELECT 1;
-- Each failure is followed by a complete statement with an independently checkable violation.
SELECT 1 FROM orders WHERE id::text COLLATE "C" =;
SELECT * FROM orders WHERE created_at > $1 AND id::text = $2 LIMIT 1;
SELECT 1 FROM orders WHERE id::text COLLATE "C" =;
SELECT * FROM orders WHERE created_at > $1 AND id::text = $2 LIMIT 1;
SELECT id FROM orders;
