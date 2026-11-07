SELECT * FROM topic_posts p WHERE $1 = p.topic_id ORDER BY id LIMIT $2;
SELECT * FROM topic_posts WHERE id = $1 LIMIT 5;
