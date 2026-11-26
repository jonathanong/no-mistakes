-- no-mistakes-disable-next-line postgres-explain-write-rollback: isolated sandbox
EXPLAIN ANALYZE UPDATE sample_jobs SET status = 'ready' WHERE id = 42;
EXPLAIN ANALYZE DELETE FROM sample_jobs WHERE id = 42; -- no-mistakes-disable-line postgres-explain-write-rollback: isolated sandbox
EXPLAIN ANALYZE INSERT INTO sample_jobs (id) VALUES (42);
