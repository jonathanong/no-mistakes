WITH RECURSIVE descendants(id) AS (
  SELECT id FROM accounts WHERE id = ANY($1::uuid[])
  UNION ALL
  SELECT child.id FROM accounts child
  JOIN orders edge ON edge.account_id = child.id
  JOIN descendants parent ON edge.id = parent.id
)
SELECT id FROM descendants;
SELECT a.id FROM accounts a JOIN orders o ON o.account_id = a.id;
