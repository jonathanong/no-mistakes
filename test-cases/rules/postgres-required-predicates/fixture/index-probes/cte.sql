WITH page AS (SELECT * FROM topic_posts WHERE topic_id = $1 LIMIT 5) SELECT * FROM page;
WITH page AS (SELECT * FROM topic_posts WHERE topic_id = $1) SELECT * FROM page LIMIT 5;
