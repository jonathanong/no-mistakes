WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows
), eligible AS (
  (SELECT token.id, token.replaced_by_id FROM locked AS token LIMIT 1)
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN locked AS successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id, owner_id FROM token_rows
), eligible AS (
  SELECT DISTINCT ON (token.owner_id) token.id, token.replaced_by_id
  FROM locked AS token WHERE token.owner_id = $1
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN locked AS successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT gen_random_uuid(), successor.replaced_by_id
  FROM eligible AS predecessor JOIN locked AS successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), other_rows AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows
), eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN other_rows AS successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), eligible(key_id, parent_id) AS (
  SELECT token.id AS original_id, token.replaced_by_id AS original_parent FROM locked AS token
  UNION ALL
  SELECT successor.id AS next_id, successor.replaced_by_id AS next_parent
  FROM eligible AS predecessor JOIN locked AS successor ON successor.id = predecessor.parent_id
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.key_id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), eligible AS (
  SELECT id, replaced_by_id FROM locked
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN locked AS successor ON successor.id = predecessor.replaced_by_id
), carried AS (
  SELECT node.id FROM eligible AS node
)
DELETE FROM token_rows AS target USING carried WHERE target.id = carried.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), eligible(id, noise) AS (
  SELECT token.id, gen_random_uuid() FROM locked AS token
  UNION ALL
  SELECT successor.id, gen_random_uuid()
  FROM eligible AS predecessor JOIN locked AS successor ON successor.id = predecessor.id
)
DELETE FROM token_rows AS target USING eligible AS e(noise, id) WHERE target.id = e.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows WHERE id = ANY($1::uuid[])
), eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN locked AS successor ON successor.id = predecessor.replaced_by_id
)
SELECT node.id FROM eligible AS node;
