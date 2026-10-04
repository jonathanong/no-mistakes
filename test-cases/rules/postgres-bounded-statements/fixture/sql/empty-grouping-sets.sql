-- Empty grouping sets synthesize rows despite WHERE false, then ORDER BY expands them.
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS (()) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS ((id), ()) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY ROLLUP(id) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY CUBE(id) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY ROLLUP(id), CUBE(created_at) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS (ROLLUP(id)) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS (CUBE(id)) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY () ORDER BY generate_series(1, 1000000);
-- An ordinary grouping key removes the empty set from the Cartesian product.
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS ((id)) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY id, ROLLUP(created_at) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS (()), id ORDER BY generate_series(1, 1000000);
-- A literal LIMIT still explicitly bounds a synthesized and expanded group.
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS (()) ORDER BY generate_series(1, 1000000) LIMIT 1;
-- Quoted or qualified ordinary functions are grouping expressions, not grouping constructs.
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS ((app.rollup(id))) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS (("rollup"(id))) ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders WHERE false GROUP BY GROUPING SETS ((abs(id))) ORDER BY generate_series(1, 1000000);
