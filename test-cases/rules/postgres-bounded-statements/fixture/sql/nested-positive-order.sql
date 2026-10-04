-- A LIMIT in a parenthesized SELECT still caps PostgreSQL's ORDER BY expansion.
(SELECT count(*) FROM orders WHERE false LIMIT 1) ORDER BY generate_series(1, 1000000);
((SELECT count(*) FROM orders LIMIT $1)) ORDER BY generate_series(1, 1000000);
(SELECT count(*) FROM orders FETCH FIRST 2 ROWS ONLY) ORDER BY generate_series(1, 1000000);
(SELECT count(*) FROM orders LIMIT NULL) ORDER BY generate_series(1, 1000000);
(SELECT count(*) FROM orders) ORDER BY generate_series(1, 1000000);
(SELECT count(*) FROM orders LIMIT (SELECT count(*) FROM accounts)) ORDER BY generate_series(1, 1000000);
(SELECT count(*) FROM orders LIMIT 1) UNION ALL SELECT id FROM accounts ORDER BY generate_series(1, 1000000);
SELECT count(*) FROM orders ORDER BY generate_series(1, 1000000);
