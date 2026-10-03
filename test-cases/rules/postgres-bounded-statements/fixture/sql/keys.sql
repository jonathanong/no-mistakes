-- Bounded: a unique constraint, and the whole composite primary key.
SELECT 1 FROM accounts WHERE email = $1;
SELECT 1 FROM order_lines WHERE order_id = $1 AND line_no = $2;
SELECT 1 FROM orders WHERE id IN (SELECT id FROM orders ORDER BY id LIMIT 5);
-- Unbounded: part of a composite key.
SELECT 1 FROM order_lines WHERE order_id = $1;
-- Unbounded: a partial, invalid, deferrable or expression index proves no uniqueness.
SELECT 1 FROM partial_keys WHERE slug = $1;
SELECT 1 FROM invalid_keys WHERE slug = $1;
SELECT 1 FROM deferred_keys WHERE slug = $1;
SELECT 1 FROM expr_keys WHERE lower(slug) = $1;
-- Unbounded: ranges and alternatives are not equality on the key.
SELECT 1 FROM orders WHERE id > $1;
SELECT 1 FROM orders WHERE id = $1 OR id = $2;
