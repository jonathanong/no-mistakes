SELECT id, user_id
FROM bindings
WHERE lineage_id = $1
  AND kind = $2
  AND user_id = $3
  AND released_at IS NULL;
