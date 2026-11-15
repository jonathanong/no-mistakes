DELETE FROM bindings AS b
USING flows AS f
WHERE b.lineage_id = f.device_id
  AND b.user_id = f.session_id
  AND b.kind = $1
  AND b.released_at IS NULL;
DELETE FROM bindings AS b
USING flows AS f
WHERE b.lineage_id = f.device_id
  AND b.user_id = f.session_id
  AND b.kind = $1
  AND b.released_at IS NULL
  AND f.device_id = $2
  AND f.session_id = $3
  AND f.status IN ('pending', 'ready');
