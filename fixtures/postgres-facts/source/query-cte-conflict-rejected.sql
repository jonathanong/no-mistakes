-- PostgreSQL-valid target predicates exceed the shared parser's nested INSERT support.
WITH a AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) WHERE id > 0 DO NOTHING RETURNING id
)
SELECT id FROM a;
SELECT 42 AS neighbor;
