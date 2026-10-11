INSERT INTO archive SELECT * FROM live
WITH changed AS (UPDATE items SET note = NULL RETURNING *) SELECT * FROM changed
WITH changed AS (INSERT INTO items (note) VALUES (NULL) RETURNING *) DELETE FROM logs
UPDATE logs SET note = 'select' /* SELECT */
SELECT 'update', $$insert$$, "merge" -- UPDATE
