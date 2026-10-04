-- DISTINCT ON evaluates its key SRF after the implicit aggregate group is formed.
SELECT DISTINCT ON (generate_series(1, count(*))) 1
FROM orders
HAVING true;
-- A DISTINCT ON key without a set-returning function still permits the implicit-group cap.
SELECT DISTINCT ON (1) count(*)
FROM orders
HAVING true;
