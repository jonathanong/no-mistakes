SELECT i.id FROM items i JOIN users u ON u.tenant_id = i.tenant_id;

SELECT i.id FROM items i JOIN users u ON u.id = coalesce(i.created_by_id, i.id);

SELECT i.id FROM items i JOIN users u ON u.id = i.created_by_id OR u.handle = i.handle;

SELECT i.id FROM items i JOIN users u ON u.email = i.email;

WITH RECURSIVE walk(id, parent_id) AS (
  SELECT i.id, i.parent_id FROM items i WHERE i.id = $1
  UNION ALL
  SELECT i.id, i.parent_id FROM items i JOIN walk w ON i.id = w.parent_id
)
SELECT * FROM walk;

SELECT i.id FROM items i CROSS JOIN users u
WHERE u.id = i.created_by_id OR u.id = i.id;

SELECT i.id FROM items i JOIN users u ON u.id = i.created_by_id
WHERE u.deleted_at IS NULL OR u.email = i.email;
