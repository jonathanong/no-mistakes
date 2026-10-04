-- Parenthesized queries and set operations expose their leftmost output names.
SELECT id FROM ((SELECT status, id FROM orders)) AS q(kind);
SELECT id FROM (
  SELECT status, id FROM orders
  UNION ALL
  SELECT status, id FROM orders
) AS q(kind);
SELECT kind FROM (VALUES (1, 2)) AS q(kind, id);
-- A wildcard before the renamed prefix has an unknown number of output columns.
SELECT id FROM (SELECT *, id FROM orders) AS q(kind);
-- The remaining expression has no known output name.
SELECT id FROM (SELECT status, 1 + 1 FROM orders) AS q(kind);
SELECT id FROM (VALUES (1, 2)) AS q(kind);
-- An invalid alias list longer than the projection remains unknown.
SELECT key FROM (SELECT id FROM orders) AS q(key, extra);
