SELECT set_config('default_transaction_read_only', 'on', true);
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;

BEGIN;
SELECT set_config('default_transaction_read_only', 'on', true);
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
COMMIT;

BEGIN;
SELECT set_config('transaction_read_only', 'on', true);
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
COMMIT;
EXPLAIN ANALYZE DELETE FROM sample_jobs WHERE id = 42;

BEGIN;
SELECT set_config('default_transaction_read_only', 'on', false);
ROLLBACK;
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;

SELECT set_config('default_transaction_read_only', 'on', false);
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;

BEGIN;
EXPLAIN ANALYZE DELETE FROM sample_jobs WHERE id = 42;
COMMIT;

BEGIN READ WRITE;
EXPLAIN ANALYZE INSERT INTO sample_jobs (id) VALUES (42);
COMMIT;

SELECT pg_catalog.set_config('default_transaction_read_only', 'off', false);
EXPLAIN ANALYZE DELETE FROM sample_jobs WHERE id = 43;
