WITH RECURSIVE walk(item_id) AS (
  SELECT i.id FROM items i WHERE i.id = $1
  UNION ALL
  SELECT DISTINCT ON (i.url_id) i.id FROM items i
  JOIN walk w ON i.url_id = w.item_id WHERE i.url_id = $2
)
SELECT item_id FROM walk;
