WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < $2
) SELECT id FROM walk;
WITH RECURSIVE walk AS (
  SELECT id, 0 AS depth FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5 OR a.name = 'bypass'
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.name = w.id::text WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a CROSS JOIN walk w WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id CROSS JOIN orders o WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE 5 >= w.depth AND a.id = w.id
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth <= $2
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 0 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < length(a.name)
) SELECT id FROM walk;
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, NULL FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5
) SELECT id FROM walk;
