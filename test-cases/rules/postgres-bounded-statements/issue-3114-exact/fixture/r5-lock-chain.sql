WITH lock_cte AS MATERIALIZED (
  SELECT 1 FROM record_rows r JOIN change_rows c
  ON c.tenant_id = r.tenant_id AND c.entry_id = r.id
  WHERE c.tenant_id = $1 AND c.entry_id = $2 AND c.token = $3
  FOR UPDATE OF r
), done AS (
  UPDATE change_rows SET state = 1 FROM lock_cte
  WHERE tenant_id = $1 AND entry_id = $2 AND token = $3
  RETURNING tenant_id, entry_id
), pushed AS (
  UPDATE record_rows r SET pushed_at = now() FROM done
  WHERE r.tenant_id = done.tenant_id AND r.id = done.entry_id
) SELECT EXISTS (SELECT 1 FROM done);

WITH lock_cte AS MATERIALIZED (
  SELECT 1 FROM record_rows r JOIN change_rows c
  ON c.tenant_id = r.tenant_id AND c.entry_id = r.id
  WHERE c.tenant_id = $1 AND c.entry_id = $2 AND c.token = $3
  FOR UPDATE OF r
), done AS (
  UPDATE change_rows SET state = 1 FROM lock_cte
  WHERE tenant_id = $1 AND token = $3
  RETURNING tenant_id, entry_id
), pushed AS (
  UPDATE record_rows r SET pushed_at = now() FROM done
  WHERE r.tenant_id = done.tenant_id AND r.id = done.entry_id
) SELECT EXISTS (SELECT 1 FROM done);
