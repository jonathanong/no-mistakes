-- no-mistakes-disable-file postgres-explain-write-rollback: isolated sandbox
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
