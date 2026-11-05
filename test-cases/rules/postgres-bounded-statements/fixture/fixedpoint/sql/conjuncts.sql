SELECT i.id
FROM items i
JOIN users u ON u.id = i.created_by_id AND u.deleted_at IS NULL;

SELECT i.id
FROM items i
JOIN users u ON u.id = i.created_by_id AND u.deleted_at IS NULL
WHERE NOT EXISTS (SELECT 1 FROM blocks b WHERE b.user_id = u.id);

SELECT i.id
FROM items i
JOIN users u ON u.id = i.created_by_id AND u.deleted_at IS NULL
WHERE NOT EXISTS (SELECT 1 FROM blocks b WHERE b.user_id = u.id)
UNION
SELECT i.id
FROM items i
JOIN users u ON u.id = i.created_by_id AND u.deleted_at IS NULL;

SELECT i.id
FROM items i
JOIN feeds f ON f.id = i.feed_id
JOIN topics t ON t.id = f.topic_id
JOIN users u ON u.id = t.created_by_id AND u.deleted_at IS NULL
WHERE NOT EXISTS (SELECT 1 FROM blocks b WHERE b.user_id = u.id);
