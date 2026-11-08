WITH done AS (
  UPDATE change_rows c SET value = $3
  WHERE c.tenant_id = $1 AND c.entry_id = $2
  RETURNING c.tenant_id, c.entry_id
), pushed AS (
  UPDATE record_rows r SET value = $3 FROM done
  WHERE r.tenant_id = done.tenant_id AND r.id = done.entry_id
) SELECT 1;

WITH gate AS (SELECT pg_advisory_xact_lock(1))
UPDATE record_rows r SET value = $3 FROM gate
WHERE r.tenant_id = $1 AND r.id = $2;

WITH gate AS (SELECT pg_advisory_xact_lock(1))
UPDATE record_rows SET value = $3 FROM gate
WHERE tenant_id = $1 AND id = $2;

WITH gate AS (SELECT c.entry_id FROM change_rows c LIMIT 1)
UPDATE record_rows SET value = $3 FROM gate
WHERE tenant_id = $1 AND id = $2;

WITH gate AS (SELECT 1 AS marker LIMIT 1)
UPDATE record_rows SET value = $3 FROM gate
WHERE tenant_id = $1 AND id = $2;

WITH done AS (
  UPDATE change_rows c SET value = $3 WHERE c.tenant_id = $1
  RETURNING c.tenant_id, c.entry_id
), pushed AS (
  UPDATE record_rows r SET value = $3 FROM done
  WHERE r.tenant_id = done.tenant_id AND r.id = done.entry_id
) SELECT 1;

WITH done AS (
  UPDATE change_rows c SET value = $3 WHERE c.tenant_id = $1 AND c.entry_id = $2
  RETURNING c.tenant_id, c.entry_id
), pushed AS (
  UPDATE record_rows r SET value = $3 FROM done
  WHERE r.tenant_id = done.tenant_id
) SELECT 1;

WITH done AS (
  UPDATE change_rows c SET value = $3 WHERE c.tenant_id = $1 AND c.entry_id = $2
  RETURNING c.tenant_id, c.entry_id
), pushed AS (
  UPDATE record_rows r SET value = $3 FROM done
  WHERE r.tenant_id = done.tenant_id AND id = done.entry_id
) SELECT 1;

WITH gate AS (SELECT pg_advisory_xact_lock(1)), done AS (
  UPDATE change_rows SET value = $3 FROM gate
  WHERE tenant_id = $1 AND entry_id = $2
  RETURNING tenant_id, entry_id
), pushed AS (
  UPDATE record_rows r SET value = $3 FROM done
  WHERE r.tenant_id = done.tenant_id AND r.id = done.entry_id
) SELECT 1;

WITH gate AS (SELECT 1 AS id LIMIT 1)
UPDATE record_rows SET value = $3 FROM gate
WHERE tenant_id = $1 AND id = $2;

WITH gate AS (SELECT * FROM change_rows LIMIT 1)
UPDATE record_rows SET value = $3 FROM gate
WHERE tenant_id = $1 AND id = $2;

WITH gate AS (SELECT 1 AS marker LIMIT 1)
UPDATE record_rows SET value = $3 FROM gate
WHERE tenant_id = $1;
