SELECT id FROM orders WHERE $1::boolean IS NOT NULL AND id > $2 ORDER BY id LIMIT $3;
SELECT id FROM orders WHERE $1::boolean IS NOT NULL AND id > $2 AND deleted_at IS NULL ORDER BY id LIMIT $3;
