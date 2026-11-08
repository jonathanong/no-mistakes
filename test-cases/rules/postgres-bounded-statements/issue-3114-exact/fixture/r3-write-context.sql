WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
SELECT target.id FROM token_rows target JOIN eligible ON target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
UPDATE token_rows target SET owner_id = $2 FROM eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows target USING eligible WHERE target.id = eligible.id;

WITH changed AS (
WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows target USING eligible WHERE target.id = eligible.id RETURNING target.id
)
SELECT changed.id FROM changed;
