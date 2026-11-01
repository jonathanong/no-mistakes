WITH page AS MATERIALIZED (
  SELECT m.owner_id, m.id FROM messages m
  WHERE m.source_id <= $1 AND m.owner_id >= $2::uuid
    AND (m.source_id, m.owner_id, m.id) >= ($1::uuid, $2::uuid, $3::uuid)
  ORDER BY m.source_id, m.owner_id, m.id
  LIMIT $5
  FOR UPDATE OF m
), changed AS (
  UPDATE messages m SET deleted_at = CASE WHEN $4 THEN now() ELSE NULL END
  FROM page
  WHERE m.owner_id = ANY (ARRAY[page.owner_id]) AND m.id = ANY (ARRAY[page.id]) AND $5
  RETURNING m.owner_id, m.id, m.deleted_at
)
SELECT page.owner_id, page.id, changed.id, changed.deleted_at
FROM page LEFT JOIN changed USING (owner_id, id)
ORDER BY page.owner_id, page.id
