-- no-mistakes-disable-next-line postgres-sql-shape-policy: one-off inventory
SELECT 1
WHERE EXISTS (
  SELECT 1 FROM topics
  UNION ALL
  SELECT 1 FROM topics
);
