UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status = 'pending';
