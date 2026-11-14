UPDATE external_orders SET updated_at = now();
-- The pre-existing column order is unknown, so this positional insert is unanalyzable.
INSERT INTO external_orders VALUES (1);
