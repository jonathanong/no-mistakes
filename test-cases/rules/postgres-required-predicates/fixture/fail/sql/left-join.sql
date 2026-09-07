SELECT posts.id
FROM posts
LEFT JOIN topics ON topics.id = posts.topic_id;
