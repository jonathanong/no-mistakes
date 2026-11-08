WITH RECURSIVE ancestors AS (
  SELECT id, parent_id, 1 AS depth FROM comments WHERE id = $1
  UNION ALL
  SELECT c.id, c.parent_id, a.depth + 1
  FROM comments c JOIN ancestors a ON c.id = a.parent_id
  WHERE a.depth < 50
) SELECT * FROM ancestors;

WITH RECURSIVE ancestors AS (
  SELECT id, parent_id, 1 AS depth FROM comments WHERE id = $1
  UNION ALL
  SELECT c.id, c.parent_id, a.depth + 1
  FROM comments c JOIN ancestors a ON c.id = a.parent_id
) SELECT * FROM ancestors;

WITH RECURSIVE ancestors AS (
  SELECT id, parent_id, 1 AS depth FROM comments WHERE id = $1
  UNION ALL
  SELECT c.id, c.parent_id, a.depth + 1
  FROM comments c JOIN ancestors a ON c.id = a.parent_id
  WHERE a.depth < 50 OR c.parent_id = $2
) SELECT * FROM ancestors;

WITH RECURSIVE ancestors AS (
  SELECT id, parent_id, 1 AS depth FROM comments WHERE parent_id = $1
  UNION ALL
  SELECT c.id, c.parent_id, a.depth + 1
  FROM comments c JOIN ancestors a ON c.id = a.parent_id
  WHERE a.depth < 50
) SELECT * FROM ancestors;

WITH RECURSIVE ancestors AS (
  SELECT id, parent_id, 1 AS depth FROM comments WHERE id = $1
  UNION ALL
  SELECT c.id, c.parent_id, a.depth + 1
  FROM comments c JOIN ancestors a ON c.parent_id = a.id
  WHERE a.depth < 50
) SELECT * FROM ancestors;

WITH RECURSIVE ancestors AS (
  SELECT id, parent_id, 1 AS depth FROM comments WHERE id = $1
  UNION ALL
  SELECT c.id, c.parent_id, a.depth
  FROM comments c JOIN ancestors a ON c.id = a.parent_id
  WHERE a.depth < 50
) SELECT * FROM ancestors;
