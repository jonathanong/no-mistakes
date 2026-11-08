WITH RECURSIVE locked AS NOT MATERIALIZED (SELECT id, replaced_by_id FROM token_rows LIMIT 1),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS (SELECT id, replaced_by_id FROM token_rows LIMIT 1),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (SELECT id, replaced_by_id FROM token_rows LIMIT 1),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows target USING eligible WHERE target.id = eligible.id;

WITH locked AS NOT MATERIALIZED (SELECT id, replaced_by_id FROM token_rows LIMIT 1)
DELETE FROM token_rows target USING (
WITH RECURSIVE eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
SELECT id FROM eligible
) e WHERE target.id = e.id;

WITH locked AS MATERIALIZED (SELECT id, replaced_by_id FROM token_rows LIMIT 1)
DELETE FROM token_rows target USING (
WITH RECURSIVE eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
SELECT id FROM eligible
) e WHERE target.id = e.id;
