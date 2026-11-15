SELECT flows.id
FROM flows, notes
WHERE notes.id = $3
  AND device_id = $1
  AND session_id = $2
  AND flows.status IN ('pending', 'ready');
