BEGIN ISOLATION LEVEL SERIALIZABLE;
SAVEPOINT "QuotedSp";
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
ROLLBACK TO SAVEPOINT "QuotedSp";
COMMIT;

BEGIN READ ONLY;
SET transaction_read_only = off;
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
COMMIT;

SET TIME ZONE 'UTC';
SET LOCAL default_transaction_read_only = on;
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
SELECT set_config('transaction_read_only', 'on', true) AS ignored_outside_tx;
SELECT (set_config('default_transaction_read_only', 'on', false)) LIMIT 1;
VALUES (1);
WITH c AS (SELECT 1) SELECT * FROM c;
SELECT set_config('default_transaction_read_only', 'on', false) FROM sample_jobs WHERE id = 1;
EXPLAIN ANALYZE DELETE FROM sample_jobs WHERE id = 42;
