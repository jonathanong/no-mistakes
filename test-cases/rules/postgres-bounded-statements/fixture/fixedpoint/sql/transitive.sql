SELECT i.id
FROM items i
JOIN feeds f ON f.id = i.feed_id
JOIN topics t ON t.id = f.topic_id;

SELECT i.id
FROM topics t
JOIN feeds f ON t.id = f.topic_id
JOIN items i ON f.id = i.feed_id;

SELECT i.id
FROM items i
JOIN feeds f ON f.id = i.feed_id
JOIN topics t ON t.id = f.topic_id AND f.alt_id = t.id;

SELECT i.id
FROM items i
JOIN feeds f ON f.id = i.feed_id
JOIN topics t ON t.id = f.topic_id AND f.id = t.created_by_id;

SELECT i.id
FROM items i
CROSS JOIN (SELECT * FROM app.rows()) unknown_rows
JOIN feeds f ON f.id = i.feed_id;
