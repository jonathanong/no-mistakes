SELECT 1 FROM posts
WHERE EXISTS (
  SELECT 1 FROM topics WHERE topics.post_id = posts.id AND topics.id = $1
  UNION ALL
  SELECT 1 FROM tags WHERE tags.post_id = posts.id AND tags.id = $1
);
