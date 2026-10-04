SELECT id, payload
FROM events
WHERE tenant_id = $1
  AND payload ->> 'fetch' = 'FETCH FIRST (ignored) ROWS ONLY'
ORDER BY created_at DESC, id DESC
LIMIT 50;
