SELECT * FROM topic_posts WHERE (topic_id = $1 OR topic_id = $2) AND id > $3 LIMIT 5;
SELECT * FROM topic_posts WHERE topic_id = $1 OR id = $2 LIMIT 5;
SELECT * FROM topic_posts WHERE NOT (topic_id = $1) LIMIT 5;
