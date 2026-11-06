WITH selected_item AS (
  SELECT DISTINCT ON (i.item_id) i.id, i.item_id
  FROM items i
  WHERE i.item_id = $1
  ORDER BY i.item_id
)
SELECT t.id
FROM selected_item i
JOIN sources s ON s.item_id = i.item_id
JOIN feeds f ON f.id = s.feed_id AND f.deleted_at IS NULL
JOIN topics t ON t.id = f.topic_id AND t.deleted_at IS NULL
