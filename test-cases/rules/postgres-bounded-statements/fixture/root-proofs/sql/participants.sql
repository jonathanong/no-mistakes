SELECT p.user_id
FROM participants p
JOIN conversations c
  ON c.id = p.conversation_id
 AND c.kind = $2
 AND c.deleted_at IS NULL
WHERE p.conversation_id = $1;
