WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows AS token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN locked AS successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows AS token WHERE token.id = ANY($1::uuid[])
),
eligible AS (SELECT token.id, token.replaced_by_id FROM locked AS token)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows AS token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN token_rows AS successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows AS token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  LEFT JOIN LATERAL (SELECT check_row.id FROM token_rows check_row WHERE check_row.id = token.id LIMIT 1) blocker ON true
  UNION ALL
  SELECT successor.id, successor.replaced_by_id FROM eligible AS predecessor
  JOIN locked AS successor ON successor.id = predecessor.replaced_by_id
  LEFT JOIN LATERAL (SELECT check_row.id FROM token_rows check_row WHERE check_row.id = successor.id LIMIT 1) blocker ON true
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE available AS MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows
),
eligible AS (
  (SELECT token.id, token.replaced_by_id FROM available AS token LIMIT 1)
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN available AS successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows AS token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN locked AS successor ON successor.id = predecessor.replaced_by_id
)
SELECT target.id FROM token_rows AS target JOIN eligible ON target.id = eligible.id;

WITH RECURSIVE available AS NOT MATERIALIZED (
  SELECT id, replaced_by_id FROM token_rows TABLESAMPLE BERNOULLI (10) LIMIT 1
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM available AS token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id
  FROM eligible AS predecessor JOIN available AS successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows AS target USING eligible WHERE target.id = eligible.id;
