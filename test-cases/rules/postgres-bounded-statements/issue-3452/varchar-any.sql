DELETE FROM labels
WHERE device_id = $1
  AND session_id = $2
  AND label IN ('pending', 'ready');
