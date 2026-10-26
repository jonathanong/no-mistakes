-- The first three statements plan a call but do not execute it.
EXPLAIN SELECT pg_sleep(1);
EXPLAIN (ANALYZE false) SELECT pg_sleep(1);
EXPLAIN (ANALYZE off) SELECT pg_sleep(1);
EXPLAIN ANALYZE SELECT pg_sleep(1);
EXPLAIN (ANALYZE true) SELECT pg_sleep(1);
EXPLAIN (ANALYZE on) SELECT pg_sleep(1);
