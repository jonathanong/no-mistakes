SELECT i.id
FROM items i
JOIN feeds f ON f.id = i.feed_id AND f.deleted_at IS NULL
JOIN topics t ON t.id = f.topic_id AND t.deleted_at IS NULL;
