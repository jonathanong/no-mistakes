SELECT p.id
FROM participants p
JOIN sources s ON s.item_id = p.user_id
JOIN posts x ON x.id = s.post_id
WHERE p.conversation_id = $1
