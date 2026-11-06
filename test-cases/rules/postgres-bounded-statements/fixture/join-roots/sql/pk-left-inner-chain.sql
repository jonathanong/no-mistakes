SELECT i.id
FROM genericitems i
LEFT JOIN topics t ON t.id = i.topic_id;

SELECT p.id
FROM posts p
JOIN users u ON u.id = p.user_id;

SELECT p.id
FROM posts p
JOIN topics t ON t.id = p.topic_id
JOIN users u ON u.id = p.user_id
WHERE p.id IS NOT NULL
ORDER BY p.id;


SELECT i.id
FROM genericitems i
JOIN topics t ON t.id = i.topic_id
JOIN users u ON u.id = t.owner_id;
