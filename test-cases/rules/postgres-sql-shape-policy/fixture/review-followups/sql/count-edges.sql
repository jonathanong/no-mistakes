SELECT (SELECT COUNT(*) AS n FROM orders) > 0;
SELECT id < (SELECT COUNT(*) FROM orders) FROM accounts;
SELECT COUNT(SELECT id FROM orders) > 0;
