WITH a AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) WHERE id > 0 DO NOTHING RETURNING id
)
SELECT id FROM a;
SELECT 42 AS neighbor;

WITH written AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) WHERE id > 0 DO UPDATE SET id = EXCLUDED.id WHERE id < 9
 RETURNING id
), seen AS (
 SELECT id FROM written
)
SELECT id FROM seen;

WITH plain AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) WHERE id > 0 DO NOTHING
)
SELECT id FROM plain;

WITH joined AS (
 INSERT INTO target (id)
 SELECT left_table.id FROM left_table JOIN right_table ON left_table.id = right_table.id
 ON CONFLICT (id) WHERE id > 0 DO NOTHING
 RETURNING id
)
SELECT id FROM joined;
