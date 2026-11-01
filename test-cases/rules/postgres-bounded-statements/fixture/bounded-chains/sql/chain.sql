WITH cand AS (
  SELECT g.id AS grant_id, g.source_id, g.created_at, s.id AS sub_id
  FROM grants g
  JOIN subs s ON s.source_id = g.source_id
  JOIN states st ON st.source_id = g.source_id
  WHERE g.user_id = $1 AND g.source_id = $2
    AND NOT EXISTS (SELECT 1 FROM periods p WHERE p.grant_id = g.id AND p.ended_at IS NULL)
  LIMIT 1
), act AS (
  INSERT INTO periods (grant_id, user_id, started_at)
  SELECT grant_id, $1, GREATEST($3, created_at) FROM cand RETURNING grant_id, started_at
), st AS (
  UPDATE states st SET effective_at = act.started_at
  FROM cand JOIN act ON act.grant_id = cand.grant_id
  WHERE st.source_id = cand.source_id RETURNING cand.sub_id
)
UPDATE subs s SET active = true FROM st WHERE s.id = st.sub_id RETURNING s.id
