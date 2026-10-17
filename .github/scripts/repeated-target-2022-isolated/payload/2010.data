-- A write target is physical even when a CTE has the same unqualified name.
WITH target AS (SELECT id FROM base), writes AS (UPDATE target SET id = 4 RETURNING id)
SELECT 1;
WITH target AS (SELECT id FROM base), writes AS (DELETE FROM target RETURNING id)
SELECT 1;
WITH target AS (SELECT id FROM base), writes AS (
 MERGE INTO target t USING source s ON t.id = s.id WHEN MATCHED THEN DELETE
)
SELECT 1;
WITH target AS (SELECT id FROM base), writes AS (INSERT INTO target (id) VALUES (5) RETURNING id)
SELECT 1;
