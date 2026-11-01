WITH RECURSIVE walk(id, depth) AS (
  VALUES ($1::uuid, 0)
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk AS (
  SELECT a.*, 0 AS depth FROM accounts a WHERE id = $1
  UNION ALL
  SELECT a.*, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk AS (
  SELECT a.*, 0 AS depth FROM accounts a WHERE id = $1
  UNION ALL
  SELECT a.id, a.email, a.name, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE a.id = w.id AND w.depth < 5
) SELECT id FROM walk;
