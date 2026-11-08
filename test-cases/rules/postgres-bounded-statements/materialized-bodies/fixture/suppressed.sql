-- no-mistakes-disable-next-line postgres-bounded-statements
WITH candidates AS MATERIALIZED (SELECT id FROM orders)
SELECT id FROM candidates LIMIT 20;

SELECT id FROM (SELECT id FROM orders) candidates LIMIT 20; -- no-mistakes-disable-line postgres-bounded-statements

WITH candidates AS MATERIALIZED (SELECT id FROM orders)
SELECT id FROM candidates LIMIT 20;
