-- Both implicit FETCH clauses must retain distinct physical locations.
SELECT id FROM orders
OFFSET (
  SELECT 0 FROM x
  FETCH FIRST ROW ONLY
)
FETCH FIRST ROW ONLY;

SELECT id FROM orders
OFFSET (
  SELECT 0 FROM x
  -- no-mistakes-disable-next-line postgres-sql-shape-policy
  FETCH FIRST ROW ONLY
)
FETCH FIRST ROW ONLY;

SELECT id FROM orders
OFFSET (
  SELECT 0 FROM x
  FETCH FIRST ROW ONLY
)
-- no-mistakes-disable-next-line postgres-sql-shape-policy
FETCH FIRST ROW ONLY;

-- A parenthesized query body starts before its inner SELECT span.
(
  SELECT id FROM orders
  FETCH FIRST ROW ONLY
)
FETCH FIRST ROW ONLY;

(
  SELECT id FROM orders
  FETCH FIRST ROW ONLY
)
-- no-mistakes-disable-next-line postgres-sql-shape-policy
FETCH FIRST ROW ONLY;
