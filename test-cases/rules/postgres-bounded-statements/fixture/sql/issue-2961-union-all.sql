-- Outer pins reach each UNION ALL arm that projects the column.
SELECT 1
FROM (
  SELECT id, email FROM accounts
  UNION ALL
  SELECT id, email FROM accounts
) binding
WHERE binding.id = ANY($1) OR binding.email = ANY($2);

-- Column aliases name the same projected outputs.
SELECT 1
FROM (
  SELECT id, email FROM accounts
  UNION ALL
  SELECT id, email FROM accounts
) AS binding(id, email)
WHERE binding.id = ANY($1) OR binding.email = ANY($2);
