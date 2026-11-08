-- no-mistakes-disable-file postgres-bounded-statements
WITH candidates AS MATERIALIZED (SELECT id FROM orders)
SELECT id FROM candidates LIMIT 20;
