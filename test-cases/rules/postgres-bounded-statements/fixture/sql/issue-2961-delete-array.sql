-- A capped CTE sizes the array of key candidates.
WITH expired AS (
  SELECT id FROM exports WHERE expires_at < now() ORDER BY id LIMIT $1
)
DELETE FROM exports WHERE id = ANY(ARRAY(SELECT id FROM expired));

WITH expired AS (
  SELECT id FROM exports WHERE expires_at < now() ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED
)
DELETE FROM exports WHERE id = ANY(ARRAY(SELECT id FROM expired));

WITH batch AS (SELECT id FROM accounts LIMIT $1)
DELETE FROM accounts WHERE id = ANY(ARRAY(SELECT id FROM batch));
