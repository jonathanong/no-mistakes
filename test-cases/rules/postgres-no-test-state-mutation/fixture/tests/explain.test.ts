import { query } from '@example/db';

// Plain EXPLAIN and explicit false/off only plan the enclosed query.
query('EXPLAIN SELECT pg_sleep(1)');
query('EXPLAIN (ANALYZE false) SELECT pg_sleep(1)');
query('EXPLAIN (ANALYZE off) SELECT pg_sleep(1)');

query('EXPLAIN ANALYZE SELECT pg_sleep(1)');
query('EXPLAIN (ANALYZE true) SELECT pg_sleep(1)');
query('EXPLAIN (ANALYZE on) SELECT pg_sleep(1)');
