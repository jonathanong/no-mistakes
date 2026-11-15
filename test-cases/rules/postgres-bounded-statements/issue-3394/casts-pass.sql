UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status IN ('pending'::varchar(7), 'ready'::varchar(5));
UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status IN ('pending'::varchar, 'ready'::character varying);
UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status = ANY (ARRAY['pending', 'ready']::text[]);
UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status = ANY (ARRAY['pending', 'ready']::varchar(7)[]);
SELECT id
FROM bindings
WHERE lineage_id = $1
  AND released_at IS NULL
  AND kind = 'direct'::source_kinds;
SELECT id, user_id
FROM bindings
WHERE lineage_id = $1
  AND kind = $2
  AND released_at IS NULL
  AND ($2 = 'direct'::source_kinds OR user_id = $3);
