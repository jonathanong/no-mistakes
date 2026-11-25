-- no-mistakes-disable-file postgres-lock-reason: legacy backfill script
SELECT id FROM orders WHERE id = $1 FOR UPDATE;
LOCK TABLE orders IN SHARE MODE;
