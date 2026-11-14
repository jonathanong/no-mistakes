INSERT INTO items (a, b, col) SELECT a, b, col FROM staged
ON CONFLICT (col) WHERE col IS NOT NULL
DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b
WHERE (items.a, items.b) IS DISTINCT FROM (EXCLUDED.a, EXCLUDED.b);
