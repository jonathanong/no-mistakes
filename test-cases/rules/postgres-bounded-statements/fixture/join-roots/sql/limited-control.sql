SELECT p.id
FROM posts p
JOIN users u ON u.id = p.user_id
LIMIT 10;


SELECT p.id
FROM posts p
JOIN topics t ON t.id = p.topic_id
WHERE p.user_id IS NOT NULL
ORDER BY t.name
LIMIT 10;
