INSERT INTO items (id, value) VALUES
  (1, COALESCE('λ', lower('first'))),
  (2, CURRENT_TIMESTAMP)
ON CONFLICT (id) DO UPDATE SET value = EXCLUDED.value;
-- Branch paths retain the set tree, rather than flattening it into one SELECT.
INSERT INTO items (id, value)
SELECT 1 AS id, COALESCE('λ', lower('left')) AS value
UNION ALL
(SELECT 2, CURRENT_TIMESTAMP INTERSECT SELECT 3, upper('right'))
ON CONFLICT (id) DO UPDATE SET value = EXCLUDED.value;
INSERT INTO items (id, value)
(VALUES (4, lower('values'))) UNION ALL SELECT 5, upper('select');
-- Parentheses, CAST syntax, and unary tokens are absent from parser AST spans.
INSERT INTO items (id, value) SELECT 6, -(CAST((COALESCE(1, 2)) AS integer));
INSERT INTO items (id, value) SELECT 7, DATE '2026-10-08';
INSERT INTO items (id, value) SELECT 8, sum(1) OVER ();
INSERT INTO items (id, value) SELECT 9, -(1) + 2;
