-- Source facts preserve unsupported pieces rather than dropping the typed child.
WITH unsupported_insert AS (
 INSERT INTO target (id) SELECT id FROM generate_series(1, 2) g RETURNING id
), unsupported_merge AS (
 MERGE INTO target t USING generate_series(1, 2) s ON t.id = s.id
 WHEN NOT MATCHED THEN INSERT (id) VALUES (s.id)
 WHEN MATCHED THEN UPDATE SET id = s.id
)
SELECT 1;

WITH limited AS (INSERT INTO target (id) VALUES (1), (2) LIMIT 1 RETURNING id)
SELECT 1;
WITH ordered AS (INSERT INTO target (id) VALUES (1), (2) ORDER BY 1 RETURNING id)
SELECT 1;
WITH fetched AS (INSERT INTO target (id) VALUES (1), (2) FETCH FIRST 1 ROW ONLY RETURNING id)
SELECT 1;
