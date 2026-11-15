UPDATE flows_written
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status = ANY (ARRAY['ready'::text, 'pending'::text]);
