SELECT p.id
FROM users u
JOIN posts p ON p.user_id = u.id
WHERE u.id = $1
