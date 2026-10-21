-- An expression arbiter inside a CTE still exceeds the shared parser.
WITH a AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (lower(id)) DO NOTHING RETURNING id
)
SELECT id FROM a;
SELECT 42 AS neighbor;
