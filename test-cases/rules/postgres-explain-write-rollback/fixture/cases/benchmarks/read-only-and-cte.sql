BEGIN;
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
SET LOCAL transaction_read_only = on;
EXPLAIN ANALYZE DELETE FROM sample_jobs WHERE id = 43;
COMMIT;

BEGIN;
EXPLAIN ANALYZE INSERT INTO sample_jobs (id) VALUES (42);
SELECT set_config('transaction_read_only', 'on', true);
EXPLAIN ANALYZE DELETE FROM sample_jobs WHERE id = 43;
COMMIT;

BEGIN;
EXPLAIN ANALYZE
WITH seed AS (SELECT 45 AS id),
  retired AS (DELETE FROM sample_jobs USING seed WHERE sample_jobs.id = seed.id RETURNING sample_jobs.id)
SELECT id FROM retired;
COMMIT;

BEGIN;
SAVEPOINT before_ro;
SET LOCAL transaction_read_only = on;
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
ROLLBACK TO SAVEPOINT before_ro;
EXPLAIN ANALYZE DELETE FROM sample_jobs WHERE id = 44;
COMMIT;

BEGIN;
SAVEPOINT before_session_ro;
SET default_transaction_read_only = on;
ROLLBACK TO SAVEPOINT before_session_ro;
COMMIT;
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 46;

EXPLAIN (ANALYZE $1) UPDATE sample_jobs SET status = 'ready' WHERE id = 47;
EXPLAIN (CUSTOM_OPT TRUE) DELETE FROM sample_jobs WHERE id = 48;
EXPLAIN (ANALYZE TRUE, GENERIC_PLAN TRUE) INSERT INTO sample_jobs (id) VALUES (49);
EXPLAIN (CUSTOM_OPT TRUE) SELECT id FROM sample_jobs;
