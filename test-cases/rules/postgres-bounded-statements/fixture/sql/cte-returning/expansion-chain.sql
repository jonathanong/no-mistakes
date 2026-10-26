-- Repeated RETURNING references must share the existing conservative CTE expansion cap.
WITH u0 AS (UPDATE orders SET status = 'processed' WHERE id = $1 RETURNING id),
u1 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u0 left_input)
    AND id IN (SELECT id FROM u0 right_input)
  RETURNING id
),
u2 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u1 left_input)
    AND id IN (SELECT id FROM u1 right_input)
  RETURNING id
),
u3 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u2 left_input)
    AND id IN (SELECT id FROM u2 right_input)
  RETURNING id
),
u4 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u3 left_input)
    AND id IN (SELECT id FROM u3 right_input)
  RETURNING id
),
u5 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u4 left_input)
    AND id IN (SELECT id FROM u4 right_input)
  RETURNING id
),
u6 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u5 left_input)
    AND id IN (SELECT id FROM u5 right_input)
  RETURNING id
),
u7 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u6 left_input)
    AND id IN (SELECT id FROM u6 right_input)
  RETURNING id
),
u8 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u7 left_input)
    AND id IN (SELECT id FROM u7 right_input)
  RETURNING id
),
u9 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u8 left_input)
    AND id IN (SELECT id FROM u8 right_input)
  RETURNING id
),
u10 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u9 left_input)
    AND id IN (SELECT id FROM u9 right_input)
  RETURNING id
),
u11 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u10 left_input)
    AND id IN (SELECT id FROM u10 right_input)
  RETURNING id
),
u12 AS (
  UPDATE orders SET status = 'processed'
  WHERE id IN (SELECT id FROM u11 left_input)
    AND id IN (SELECT id FROM u11 right_input)
  RETURNING id
)
SELECT 1 FROM accounts a JOIN u12 ON a.id = u12.id;
