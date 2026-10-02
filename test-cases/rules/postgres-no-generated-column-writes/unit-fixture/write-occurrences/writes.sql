SELECT updated_at FROM orders;
-- Repeated writes must retain distinct statement locations.
UPDATE orders SET updated_at = now();

-- no-mistakes-disable-next-line postgres-no-generated-column-writes
UPDATE orders SET updated_at = now();
UPDATE orders SET updated_at = now(); -- no-mistakes-disable-line postgres-no-generated-column-writes
UPDATE orders SET updated_at = now();
