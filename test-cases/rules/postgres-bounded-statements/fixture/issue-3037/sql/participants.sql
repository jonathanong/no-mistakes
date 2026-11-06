SELECT u.id
FROM participants p
JOIN conversations c ON c.id = p.conversation_id
  AND c.kind = $2
  AND c.deleted_at IS NULL
JOIN users u ON u.id = p.user_id
  AND u.deleted_at IS NULL
WHERE p.conversation_id = $1
  AND p.removed_at IS NULL
