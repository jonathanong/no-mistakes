SELECT EXISTS (SELECT 1 FROM source_rows WHERE group_id = $1 AND user_id = $2 AND inactive_at IS NULL) AS a,
       EXISTS (SELECT 1 FROM edge_rows WHERE group_id = $1 AND inactive_at IS NULL) AS b;

SELECT k.id FROM lookup_rows k
WHERE k.key_hash = $1 AND NOT EXISTS (
  SELECT 1 FROM edge_rows e JOIN label_rows l ON l.id = e.label_id
  WHERE e.user_id = k.user_id AND l.slug = 'admin'
) LIMIT 1;

SELECT (SELECT e.id FROM edge_rows e WHERE e.user_id = k.user_id)
FROM lookup_rows k LIMIT 1;

SELECT (SELECT e.id FROM edge_rows e WHERE e.user_id = k.user_id LIMIT 1)
FROM lookup_rows k LIMIT 1;
