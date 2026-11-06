SELECT t.id
FROM participants p
JOIN sources s ON s.feed_id = p.conversation_id
JOIN topics t ON t.id = s.feed_id AND t.deleted_at IS NULL
WHERE p.conversation_id = $1;
