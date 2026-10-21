WITH broken AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) WHERE , DO NOTHING
)
SELECT id FROM broken;
SELECT 42 AS kept;

WITH open AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) WHERE id > 0
)
SELECT id FROM open;
SELECT 7 AS kept;
