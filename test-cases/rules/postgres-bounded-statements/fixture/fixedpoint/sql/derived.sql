WITH subtree AS (
  SELECT i.id AS id, i.created_by_id AS created_by_id
  FROM items i
)
SELECT s.id
FROM subtree s
JOIN users u ON u.id = s.created_by_id;

SELECT s.id
FROM (SELECT i.id AS id, i.created_by_id AS created_by_id FROM items i) s
JOIN users u ON u.id = s.created_by_id;

WITH subtree(id, parent_id, created_by_id) AS (
  SELECT i.id, i.parent_id, i.created_by_id FROM items i
  UNION ALL
  SELECT i.id, i.parent_id, i.created_by_id FROM items i
)
SELECT s.id FROM subtree s JOIN users u ON u.id = s.created_by_id;

WITH RECURSIVE subtree(id, parent_id, created_by_id) AS (
  SELECT i.id, i.parent_id, i.created_by_id FROM items i WHERE i.id = $1
  UNION ALL
  SELECT c.id, c.parent_id, c.created_by_id FROM items c JOIN subtree s ON c.parent_id = s.id
)
SELECT s.id FROM subtree s JOIN users u ON u.id = s.created_by_id;

WITH subtree AS (SELECT * FROM items)
SELECT s.id FROM subtree s JOIN users u ON u.id = s.created_by_id;

WITH projected AS (SELECT i.id AS id, coalesce(i.created_by_id, i.id) AS user_id FROM items i)
SELECT p.id FROM projected p JOIN users u ON u.id = p.user_id;
