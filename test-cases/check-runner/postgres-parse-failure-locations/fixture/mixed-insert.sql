INSERT INTO orders(id) VALUES ($1) ON CONFLICT DO NOTHING;
-- A valid INSERT before a malformed non-INSERT must not make the failure INSERT-related.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $1;
