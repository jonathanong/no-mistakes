SELECT id FROM jobs FOR UPDATE;
SELECT id FROM jobs FOR NO /* strength */ KEY UPDATE NOWAIT;
SELECT id FROM jobs FOR SHARE SKIP LOCKED LIMIT 1;
SELECT id FROM jobs FOR KEY SHARE;
SELECT a.id FROM jobs a JOIN jobs b ON a.id = b.id FOR UPDATE OF a, b NOWAIT FOR SHARE OF a;
WITH candidate AS (
    SELECT id FROM jobs FOR UPDATE
)
SELECT id FROM candidate FOR SHARE;
LOCK TABLE ONLY jobs, other_jobs IN SHARE ROW EXCLUSIVE MODE NOWAIT;
LOCK jobs;
SELECT pg_advisory_lock(1), pg_advisory_lock(1),
       pg_catalog."pg_try_advisory_xact_lock_shared"(3), pg_advisory_unlock_all();
SELECT 'FOR UPDATE NOWAIT pg_advisory_lock(1)', $$LOCK TABLE jobs$$, "pg_advisory_lock"
FROM jobs AS "FOR UPDATE";
-- FOR SHARE SKIP LOCKED
SELECT project_lock(1);
EXPLAIN SELECT id FROM jobs FOR UPDATE;
EXPLAIN ANALYZE SELECT id FROM jobs FOR UPDATE;
