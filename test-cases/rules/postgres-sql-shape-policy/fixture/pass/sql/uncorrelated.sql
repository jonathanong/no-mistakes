-- Uncorrelated EXISTS(UNION) is an InitPlan (once per query), not the ban.
SELECT 1
WHERE EXISTS (
  SELECT 1 FROM topics
  UNION ALL
  SELECT 1 FROM topics
);
