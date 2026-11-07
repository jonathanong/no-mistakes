SELECT * FROM (SELECT * FROM topic_posts WHERE topic_id = $1 LIMIT $2) p;
SELECT * FROM topic_posts WHERE id = $1 LIMIT 5;
