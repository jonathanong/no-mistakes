SELECT EXISTS (
  SELECT 1 FROM topics
  UNION ALL
  SELECT 1 FROM topics
) AS has_any;
