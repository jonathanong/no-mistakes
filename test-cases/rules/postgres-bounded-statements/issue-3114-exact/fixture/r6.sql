WITH emitted AS (
  INSERT INTO token_rows (owner_id)
  WITH RECURSIVE locked AS MATERIALIZED (
    SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
  ), eligible AS (
    SELECT token.id, token.replaced_by_id FROM locked token
    UNION ALL
    SELECT successor.id, successor.replaced_by_id
    FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
  )
  SELECT eligible.id FROM eligible
  RETURNING id
)
UPDATE token_rows target SET owner_id = $2 FROM emitted WHERE target.id = emitted.id;

WITH emitted AS (
  INSERT INTO token_rows (owner_id)
  SELECT token.id FROM token_rows token WHERE token.id = $1
  RETURNING id
)
UPDATE token_rows target SET owner_id = $2 FROM emitted WHERE target.id = emitted.id;
