-- no-mistakes-disable-next-line postgres-bounded-statements -- outer EXPLAIN owns the executed write.
EXPLAIN ANALYZE
UPDATE accounts
SET email = 'wrapped';
-- no-mistakes-disable-next-line postgres-bounded-statements -- WITH owns its later UPDATE.
WITH chosen AS (SELECT 1)
UPDATE accounts
SET email = 'with';
WITH chosen AS (SELECT 1) -- no-mistakes-disable-line postgres-bounded-statements
DELETE FROM accounts;
EXPLAIN ANALYZE
-- no-mistakes-disable-next-line postgres-bounded-statements -- inner keyword directives still work.
UPDATE accounts
SET email = 'inner';
-- no-mistakes-disable-next-line postgres-explicit-columns -- other rules do not suppress this write.
EXPLAIN ANALYZE
DELETE FROM accounts;
WITH chosen AS (SELECT 1)
UPDATE accounts
SET email = 'unsuppressed';
