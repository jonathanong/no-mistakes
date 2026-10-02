UPDATE external_orders SET updated_at = now();
-- The pre-existing column order is unknown, so do not invent positional hits.
INSERT INTO external_orders VALUES (1);
