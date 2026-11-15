SELECT id, user_id
FROM bindings
WHERE lineage_id = $1
  AND kind = $2
  AND released_at IS NULL
  AND ($2 = 'direct' OR released_at IS NULL);
