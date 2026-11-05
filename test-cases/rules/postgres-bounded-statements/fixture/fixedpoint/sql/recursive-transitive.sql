WITH RECURSIVE subtree(id, parent_id, feed_id) AS (
  SELECT i.id, i.parent_id, i.feed_id FROM items i WHERE i.id = $1
  UNION ALL
  SELECT c.id, c.parent_id, c.feed_id FROM items c JOIN subtree s ON c.parent_id = s.id
)
SELECT s.id FROM subtree s
JOIN feeds f ON f.id = s.feed_id
JOIN topics t ON t.id = f.topic_id;
