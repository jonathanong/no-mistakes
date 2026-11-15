SELECT user_id
FROM bindings
WHERE lineage_id = $1
  AND kind = 'direct'
  AND released_at IS NULL;
