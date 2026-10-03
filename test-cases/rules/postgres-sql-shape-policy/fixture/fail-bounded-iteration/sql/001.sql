SELECT id FROM orders WHERE id > $1 ORDER BY id LIMIT 500;
SELECT id FROM orders WHERE ($1::uuid IS NULL OR id > $1) AND deleted_at IS NULL ORDER BY id LIMIT $2;
WITH c AS (SELECT id FROM invoices ORDER BY id LIMIT 1000 FOR UPDATE SKIP LOCKED) DELETE FROM invoices USING c WHERE invoices.id = c.id;
SELECT id FROM events WHERE (created_at, id) > ($1, $2) ORDER BY created_at, id FETCH FIRST 50 ROWS ONLY;
