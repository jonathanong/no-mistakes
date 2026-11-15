SELECT LEFT(p.title, 80) AS label
FROM entries p
ORDER BY p.id DESC
LIMIT 26;

SELECT RIGHT(p.title, 40) AS label
FROM entries p
ORDER BY p.id DESC
LIMIT 26;

WITH candidates AS (
  SELECT flow.id AS id, make_interval(secs => $4)
  FROM flows flow
  JOIN flows current_flow ON current_flow.id = flow.id
  WHERE flow.status = 'callback_received'
    AND CASE
      WHEN $2 THEN current_flow.callback_received_at < now() - make_interval(secs => $4)
      ELSE current_flow.callback_error IS NULL
    END
  ORDER BY current_flow.exchange_attempts ASC,
    current_flow.callback_received_at ASC NULLS FIRST,
    current_flow.updated_at ASC
  LIMIT $1
)
UPDATE flows
SET status = $3,
  callback_error = NULL,
  callback_received_at = now(),
  updated_at = now()
FROM candidates
WHERE flows.id = candidates.id;

WITH candidates AS (
  SELECT flow.id AS id, make_interval(secs := $4)
  FROM flows flow
  JOIN flows current_flow ON current_flow.id = flow.id
  WHERE flow.status = 'callback_received'
    AND CASE
      WHEN $2 THEN current_flow.callback_received_at < now() - make_interval(secs := $4)
      ELSE current_flow.callback_error IS NULL
    END
  ORDER BY current_flow.exchange_attempts ASC,
    current_flow.callback_received_at ASC NULLS FIRST,
    current_flow.updated_at ASC
  LIMIT $1
)
UPDATE flows
SET status = $3,
  callback_error = NULL,
  callback_received_at = now(),
  updated_at = now()
FROM candidates
WHERE flows.id = candidates.id;

SELECT id FROM flows ORDER BY id LIMIT 25;

UPDATE flows SET status = 'x' WHERE id = $1;

SELECT 1
FROM entries e
LEFT JOIN flows f ON e.id = f.id
ORDER BY e.id
LIMIT 1;

SELECT 1
FROM entries e
RIGHT JOIN flows f ON e.id = f.id
ORDER BY e.id
LIMIT 1;
