-- A zero-row insert source is already skipped. RETURNING must not relabel it.
WITH changed AS (
  INSERT INTO orders (id)
  SELECT id FROM accounts LIMIT 0
  RETURNING id
)
SELECT id FROM changed;
