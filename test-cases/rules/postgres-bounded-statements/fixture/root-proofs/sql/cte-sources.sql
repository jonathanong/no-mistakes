WITH item AS (
  SELECT DISTINCT ON (i.url_id) i.id AS item_id
  FROM items i
  WHERE i.url_id = $1 AND i.deleted_at IS NULL
  ORDER BY i.url_id, i.id DESC
), ctx AS (
  SELECT t.id
  FROM item
  JOIN sources s ON s.item_id = item.item_id
  JOIN feeds f ON f.id = s.feed_id AND f.deleted_at IS NULL
  JOIN topics t ON t.id = f.topic_id AND t.deleted_at IS NULL
)
SELECT id FROM ctx;
