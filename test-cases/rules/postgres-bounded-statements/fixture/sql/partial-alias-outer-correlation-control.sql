-- The surviving outer alias remains correlated through a partial derived-table alias.
DELETE FROM accounts a
WHERE id IN (
  SELECT a.id
  FROM (SELECT status, id FROM orders) AS q(kind)
  LIMIT 1
);
