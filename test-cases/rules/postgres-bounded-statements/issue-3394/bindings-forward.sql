SELECT user_id
FROM bindings
WHERE lineage_id = $1
  AND kind = $2
  AND $2 = 'direct'
  AND released_at IS NULL;
