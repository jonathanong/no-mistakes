WITH visible_posts AS (
  SELECT p.id, p.user_id
  FROM posts p
  JOIN users u ON u.id = p.user_id
)
SELECT vp.id
FROM visible_posts vp
JOIN users u ON u.id = vp.user_id;

SELECT d.id
FROM (SELECT p.id, p.user_id FROM posts p) d
JOIN users u ON u.id = d.user_id;

SELECT p.id
FROM posts p
JOIN posts sibling ON sibling.id = p.id;
