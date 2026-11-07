WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows token WHERE token.id = ANY($1::uuid[])
),
joined AS (SELECT a.id FROM locked a JOIN locked b ON b.id = a.id)
DELETE FROM token_rows target USING joined WHERE target.id = joined.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token JOIN locked token ON true
  UNION ALL
  SELECT successor.id, successor.replaced_by_id FROM eligible predecessor
  JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id FROM eligible predecessor
  JOIN locked successor ON successor.id = predecessor.replaced_by_id
  LIMIT 1
)
DELETE FROM token_rows target USING eligible WHERE target.id = eligible.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id FROM eligible predecessor
  JOIN locked successor ON successor.id = predecessor.replaced_by_id
)
DELETE FROM token_rows target USING token_rows donor
CROSS JOIN LATERAL (SELECT e.id FROM eligible e WHERE e.id = donor.id) node
WHERE target.id = node.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id FROM eligible predecessor
  JOIN locked successor ON successor.id = predecessor.replaced_by_id
), carried AS (
 SELECT node.id FROM token_rows donor
 CROSS JOIN LATERAL (SELECT e.id FROM eligible e WHERE e.id = donor.id) node
)
DELETE FROM token_rows target USING carried WHERE target.id = carried.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
  SELECT token.id, token.replaced_by_id FROM locked token
  UNION ALL
  SELECT successor.id, successor.replaced_by_id FROM eligible predecessor
  JOIN locked successor ON successor.id = predecessor.replaced_by_id
), carried AS (
 SELECT side.id FROM eligible e CROSS JOIN (SELECT id FROM token_rows WHERE id = $2) side
)
DELETE FROM token_rows target USING carried WHERE target.id = carried.id;

WITH RECURSIVE locked AS MATERIALIZED (
  SELECT token.id, token.replaced_by_id FROM token_rows token WHERE token.id = ANY($1::uuid[])
),
eligible AS (
 SELECT token.id, COALESCE(token.replaced_by_id, gen_random_uuid()) FROM locked token
 UNION ALL
 SELECT successor.id, COALESCE(successor.replaced_by_id, gen_random_uuid())
 FROM eligible predecessor JOIN locked successor ON successor.id = predecessor.id
), carried AS (SELECT e.coalesce AS id FROM eligible e)
DELETE FROM token_rows target USING carried WHERE target.id = carried.id;

SELECT id FROM token_rows WHERE false;

DELETE FROM token_rows target WHERE target.id IN (SELECT id);

WITH stored AS MATERIALIZED (SELECT ARRAY(SELECT id FROM token_rows) AS ids)
DELETE FROM token_rows target USING stored WHERE target.id = ANY(stored.ids);

SELECT q.id FROM (SELECT NULL::uuid AS id FROM token_rows) q
WHERE q.id = $1 OR q.id = $2;
