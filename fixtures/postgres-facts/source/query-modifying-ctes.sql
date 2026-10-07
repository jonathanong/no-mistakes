-- Unicode and comments must survive the original child SQL slice.
WITH "Added" AS (
  INSERT INTO "Schéma"."Target" (id) VALUES (1), (2) /* kept */ RETURNING id AS "ID"
), changed AS (
  UPDATE "Schéma"."Target" AS t SET id = t.id + 1
  FROM "Added" AS a WHERE t.id = a."ID" RETURNING t.*
), removed AS (
  DELETE FROM "Schéma"."Target" AS t USING changed c WHERE t.id = c.id RETURNING *
), merged AS (
  MERGE INTO "Schéma"."Target" t USING changed c ON t.id = c.id
  WHEN MATCHED AND c.id = t.id THEN UPDATE SET id = c.id
  WHEN MATCHED THEN DELETE
  WHEN NOT MATCHED THEN INSERT (id) VALUES (c.id)
  WHEN NOT MATCHED BY SOURCE THEN DO NOTHING
  RETURNING t.id
)
SELECT "ID" FROM "Added";

-- Unreferenced modifying CTEs still execute; used remains SELECT reachability.
WITH writes AS (INSERT INTO target DEFAULT VALUES)
SELECT 1;

-- This nested syntactic form is projected, not validated as executable PostgreSQL.
WITH outer_cte AS (
  WITH inner_cte AS (DELETE FROM target RETURNING id)
  SELECT id FROM inner_cte
), final_insert AS (
  INSERT INTO target (id)
  WITH deeper AS (UPDATE target SET id = 3 RETURNING id)
  SELECT id FROM deeper RETURNING id
)
SELECT id FROM outer_cte;

WITH tuple_update AS (
  UPDATE target SET (id, name) = (4, 'name') RETURNING id, name AS label
), literals AS (
  INSERT INTO target (id) VALUES (5) ON CONFLICT (id) DO NOTHING RETURNING target.*
)
SELECT * FROM tuple_update;
