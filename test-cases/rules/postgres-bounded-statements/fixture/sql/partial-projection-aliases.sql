-- PostgreSQL renames only the first output; id remains a local column of q.
DELETE FROM accounts
WHERE id IN (
  SELECT id
  FROM (SELECT status, id FROM orders) AS q(kind)
  LIMIT 1
);
