WITH done AS (
  UPDATE change_rows c SET state = 1 WHERE c.tenant_id = $1 AND c.entry_id = $2
  RETURNING *
)
UPDATE record_rows r SET pushed_at = now() FROM done
WHERE r.tenant_id = done.tenant_id AND r.id = done.entry_id;

WITH done AS (
  UPDATE change_rows c SET state = 1 WHERE c.tenant_id = $1 AND c.entry_id = $2
  RETURNING c.entry_id::text || ''
)
UPDATE record_rows r SET pushed_at = now() FROM done
WHERE r.tenant_id = $1 AND r.id = $2;
