DELETE FROM label_lists
WHERE device_id = $1
  AND session_id = $2
  AND label IN ('pending', 'ready');
