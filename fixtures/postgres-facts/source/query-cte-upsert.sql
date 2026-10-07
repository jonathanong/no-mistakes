WITH writes AS (
 INSERT INTO target AS t (id) VALUES (1)
 ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id WHERE t.id = EXCLUDED.id
 RETURNING id
)
SELECT id FROM writes;

-- A scalar conflict expression is incomplete for INSERT provenance, but its query facts survive.
WITH source_cte AS (SELECT id FROM source), writes AS (
 INSERT INTO target (id) VALUES (2)
 ON CONFLICT (id) DO UPDATE SET id = (SELECT id FROM source_cte)
)
SELECT id FROM writes;

-- The target alias is unavailable to the source scalar query; no semantic validation is attempted.
WITH writes AS (
 INSERT INTO target AS t (id) VALUES ((SELECT t.id))
 ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id
 RETURNING t.id
)
SELECT id FROM writes;
