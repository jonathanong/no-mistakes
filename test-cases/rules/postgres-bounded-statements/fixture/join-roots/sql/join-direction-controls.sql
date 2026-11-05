SELECT i.id
FROM topics t
RIGHT JOIN genericitems i ON t.id = i.topic_id;

SELECT i.id
FROM genericitems i
FULL JOIN topics t ON t.id = i.topic_id;

SELECT p.id
FROM posts p
JOIN users u ON u.id = p.user_id OR u.email = 'someone@example.test';

SELECT i.id
FROM genericitems i
JOIN posts p ON p.id = i.id;
