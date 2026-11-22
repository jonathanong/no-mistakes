SELECT pg_catalog.pg_advisory_lock(1);
SELECT pg_catalog.pg_advisory_lock(2);
SELECT pg_catalog.pg_advisory_lock(3);
SELECT pg_catalog.pg_advisory_lock(4);
-- Alternative source layout
SELECT  pg_catalog.pg_advisory_lock(1);
SELECT pg_catalog, "pg_advisory_lock";
SELECT pg_catalog.another_lock(3), "pg_advisory_lock";
SELECT pg_catalog.pg_advisory_lock;
