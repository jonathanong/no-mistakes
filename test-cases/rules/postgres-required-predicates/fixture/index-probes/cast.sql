SELECT * FROM topic_posts WHERE topic_id = $1::integer LIMIT $2::integer;
SELECT * FROM topic_posts WHERE topic_id = id::integer LIMIT 5;
