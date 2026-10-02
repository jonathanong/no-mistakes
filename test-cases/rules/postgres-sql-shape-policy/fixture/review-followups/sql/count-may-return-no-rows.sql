-- These scalar aggregates can produce NULL when HAVING or row limits remove the row.
SELECT (SELECT COUNT(*) FROM orders HAVING false) > 0;
SELECT (SELECT COUNT(*) FROM orders LIMIT 0) > 0;
SELECT (SELECT COUNT(*) FROM orders FETCH FIRST 0 ROWS ONLY) > 0;
