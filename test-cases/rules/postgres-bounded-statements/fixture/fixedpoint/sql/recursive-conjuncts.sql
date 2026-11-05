WITH RECURSIVE subtree(id, parent_id, created_by_id) AS (
  SELECT i.id, i.parent_id, i.created_by_id FROM items i WHERE i.id = $1
  UNION ALL
  SELECT c.id, c.parent_id, c.created_by_id FROM items c JOIN subtree s ON c.parent_id = s.id
)
SELECT s.id FROM subtree s
JOIN users u ON u.id = s.created_by_id AND u.deleted_at IS NULL
WHERE NOT EXISTS (SELECT 1 FROM blocks b WHERE b.user_id = u.id)
UNION
SELECT s.id FROM subtree s
JOIN users u ON u.id = s.created_by_id AND u.deleted_at IS NULL;
