SELECT p.id
FROM posts p
JOIN LATERAL (SELECT p.id AS id) d ON p.id = d.id;

SELECT p.id
FROM posts p
JOIN external e ON p.id = e.post_id;

WITH RECURSIVE walk(id, user_id) AS (
  SELECT p.id, p.user_id
  FROM posts p
  WHERE p.id = $1
  UNION ALL
  SELECT p.id, p.user_id
  FROM posts p
  JOIN walk w ON p.id = w.user_id
)
SELECT id FROM walk;
