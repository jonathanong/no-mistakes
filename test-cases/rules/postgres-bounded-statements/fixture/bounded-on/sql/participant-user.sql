SELECT p.id
FROM participants p
JOIN users u ON u.id = p.user_id AND u.deleted_at IS NULL
WHERE p.id = $1;
