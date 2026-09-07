SELECT 1 FROM posts
WHERE
-- no-mistakes-disable-next-line postgres-sql-shape-policy: one-off inventory
EXISTS (
  SELECT 1 FROM topics WHERE topics.post_id = posts.id
  UNION ALL
  SELECT 1 FROM tags WHERE tags.post_id = posts.id
);
