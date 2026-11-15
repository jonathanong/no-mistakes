DELETE FROM notes
WHERE device_id = $1
  AND session_id = $2
  AND status = 'pending'::text;
