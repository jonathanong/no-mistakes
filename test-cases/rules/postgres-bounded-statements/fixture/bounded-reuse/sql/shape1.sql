WITH locked AS (SELECT id FROM users WHERE id = $1 FOR UPDATE),
cand AS (
  SELECT g.id, g.source_id, g.product_id, g.created_at, g.by_id, g.note, fn_remaining(g.id) AS rem
  FROM grants g JOIN subs s ON s.source_id = g.source_id JOIN states st ON st.source_id = g.source_id CROSS JOIN locked
  WHERE g.user_id = $1 AND g.revoked_at IS NULL AND ($2::uuid IS NULL OR g.source_id = $2::uuid)
    AND st.paused_at IS NULL AND fn_remaining(g.id) >= INTERVAL '1 ms'
    AND NOT EXISTS (SELECT 1 FROM periods p WHERE p.grant_id = g.id AND p.ended_at IS NULL)
  ORDER BY g.id LIMIT 1
), act AS (
  INSERT INTO periods (grant_id, user_id, started_at) SELECT id, $1, GREATEST($3, created_at) FROM cand RETURNING grant_id, started_at
), st AS (
  UPDATE states st SET effective_at = now() FROM cand INNER JOIN act ON act.grant_id = cand.id
  WHERE st.source_id = cand.source_id
  RETURNING cand.id, st.effective_at
) SELECT * FROM st
