SELECT p.id
FROM posts p
JOIN users u ON u.id = p.user_id
LIMIT 10;
