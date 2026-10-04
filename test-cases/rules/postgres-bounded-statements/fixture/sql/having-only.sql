-- HAVING introduces one implicit group even without an aggregate call.
SELECT 1 FROM orders HAVING true;
SELECT 1 FROM orders HAVING false;
-- An expanding select list or explicit grouping can still produce many rows.
SELECT generate_series(1, 10) FROM orders HAVING true;
SELECT status FROM orders GROUP BY status HAVING true;
-- ORDER BY SRFs expand the implicit group even when absent from the select list.
SELECT 1 FROM orders HAVING true ORDER BY generate_series(1, 1000000);
SELECT count(*) FROM orders HAVING true ORDER BY abs(generate_series(1, 1000000));
SELECT 1 FROM orders HAVING true ORDER BY count(*), generate_series(1, 1000000);
-- A rejecting HAVING prevents an ORDER BY SRF from expanding the implicit group.
SELECT count(*) FROM orders HAVING false ORDER BY generate_series(1, 10);
SELECT count(*) FROM orders HAVING true AND false ORDER BY generate_series(1, 10);
SELECT count(*) FROM orders HAVING false OR true ORDER BY generate_series(1, 10);
-- A window call returns one value per grouped row, even when its name matches an SRF.
SELECT count(*) FROM orders ORDER BY app.generate_series(1) OVER ();
-- An outer ORDER BY SRF can expand a bounded SELECT wrapped in parentheses.
(SELECT 1 FROM orders HAVING true) ORDER BY generate_series(1, 1000000);
-- Explicit row caps and constant-false predicates remain valid independently.
SELECT 1 FROM orders HAVING true ORDER BY generate_series(1, 1000000) LIMIT 1;
SELECT 1 FROM orders WHERE false HAVING true ORDER BY generate_series(1, 1000000);
SELECT 1 FROM orders HAVING true ORDER BY count(*);
SELECT 1 FROM orders HAVING true ORDER BY ALL;
-- The parser accepts this PostgreSQL-invalid set-operation ORDER BY expression. It has no
-- single SELECT group whose predicate can cap the outer SRF, so retain the arm findings.
SELECT 1 FROM orders HAVING false UNION ALL SELECT 1 FROM customers
ORDER BY generate_series(1, 10);
