UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status IN ('pending'::varchar(3), 'ready');
UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status = ANY (ARRAY['pending', 'ready']::varchar(3)[]);
UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status IN ('pending'::char(7), 'ready');
UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status = ANY (ARRAY['pending'::varchar(3), 'ready']::text[]);
SELECT id
FROM bindings
WHERE lineage_id = $1
  AND released_at IS NULL
  AND kind = 'nope'::source_kinds;
SELECT id
FROM bindings
WHERE lineage_id = $1
  AND released_at IS NULL
  AND kind = 'direct'::not_an_enum;
UPDATE flows
SET rejected_at = now()
WHERE device_id = $1
  AND session_id = $2
  AND status IN (('pending'::text[])::text, 'ready');
