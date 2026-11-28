-- lock-reason: writer: billing worker
SELECT id FROM invoices WHERE id = $1 FOR UPDATE;
LOCK TABLE invoices IN SHARE MODE;
SELECT pg_advisory_lock(7);
