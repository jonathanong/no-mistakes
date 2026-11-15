SELECT id
FROM quoted_kinds
WHERE lineage_id = $1
  AND released_at IS NULL
  AND kind = 'direct'::"SourceKinds";
SELECT id
FROM quoted_kinds
WHERE lineage_id = $1
  AND released_at IS NULL
  AND kind = 'nope'::"SourceKinds";
