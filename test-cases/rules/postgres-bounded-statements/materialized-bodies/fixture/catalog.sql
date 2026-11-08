WITH candidates AS MATERIALIZED (SELECT scalar_id(id) AS value FROM orders LIMIT 20)
SELECT value FROM candidates LIMIT 1;

WITH candidates AS MATERIALIZED (SELECT unknown_id(id) AS value FROM orders LIMIT 20)
SELECT value FROM candidates LIMIT 1;

SELECT (SELECT (SELECT id FROM orders)) LIMIT 1;
SELECT (SELECT (SELECT id FROM orders LIMIT 1)) LIMIT 1;
