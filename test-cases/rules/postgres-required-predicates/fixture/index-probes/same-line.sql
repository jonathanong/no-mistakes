SELECT * FROM topic_posts WHERE id = $1 AND EXISTS (SELECT 1 FROM topic_posts WHERE topic_id = $1 LIMIT 5) LIMIT 5;
