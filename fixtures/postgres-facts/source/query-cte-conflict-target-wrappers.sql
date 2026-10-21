EXPLAIN (ANALYZE) WITH explained AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) WHERE id > 0 DO NOTHING RETURNING id
)
SELECT id FROM explained;

PREPARE prepared_write AS WITH prepared AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) WHERE id > 0 DO NOTHING RETURNING id
)
SELECT id FROM prepared;
