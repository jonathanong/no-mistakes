SELECT 1;
-- no-mistakes-disable-next-line postgres-idempotent-insert
SELECT 1 FROM orders WHERE id::text COLLATE "C" =;
SELECT 1;
-- no-mistakes-disable-next-line postgres-idempotent-insert
SELECT 1 FROM orders WHERE id::text COLLATE "C" =;
