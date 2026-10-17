-- Broader dialect ASTs exercise defensive projection; public PostgreSQL rejects these forms.
WITH a AS (UPDATE target SET id = 2 RETURNING id ORDER BY id LIMIT 1) SELECT 1;
WITH a AS (DELETE FROM target RETURNING id ORDER BY id LIMIT 1) SELECT 1;
WITH a AS (INSERT INTO target (id) VALUES (1) RETURNING * EXCLUDE (id)) SELECT 1;
WITH a AS (MERGE INTO target t USING source s ON t.id = s.id
 WHEN NOT MATCHED THEN INSERT ROW
 WHEN MATCHED THEN UPDATE SET *) SELECT 1;
WITH a AS (INSERT OVERWRITE target (id) VALUES (1)) SELECT 1;
WITH a AS (MERGE INTO target t USING source s ON t.id = s.id
 WHEN MATCHED THEN DELETE OUTPUT t.id) SELECT 1;
