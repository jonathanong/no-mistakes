SELECT (SELECT COUNT(*) FROM orders) > 0;
SELECT (SELECT COUNT(*) FROM orders) = 0;
SELECT (SELECT COUNT(*) FROM orders) < 1;
SELECT (SELECT COUNT(*) FROM orders) <= 0;
SELECT 0 >= (SELECT COUNT(*) FROM orders);
SELECT pg_catalog.count(*) > 0 FROM orders;
