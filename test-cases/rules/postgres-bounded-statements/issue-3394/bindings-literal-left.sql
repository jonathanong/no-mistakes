SELECT user_id
FROM bindings
WHERE lineage_id = $1
  AND kind = $2
  AND released_at IS NULL
  AND 'direct' = $2;
