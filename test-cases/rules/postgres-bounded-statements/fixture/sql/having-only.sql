-- HAVING introduces one implicit group even without an aggregate call.
SELECT 1 FROM orders HAVING true;
SELECT 1 FROM orders HAVING false;
-- An expanding select list or explicit grouping can still produce many rows.
SELECT generate_series(1, 10) FROM orders HAVING true;
SELECT status FROM orders GROUP BY status HAVING true;
