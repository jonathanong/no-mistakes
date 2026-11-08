WITH candidates AS MATERIALIZED (SELECT id FROM orders)
SELECT id FROM candidates LIMIT 20;

WITH candidates AS MATERIALIZED (SELECT id FROM orders LIMIT 200)
SELECT id FROM candidates LIMIT 20;

WITH candidates AS MATERIALIZED (SELECT id FROM orders WHERE account_id = $1)
SELECT id FROM candidates LIMIT 20;

SELECT id FROM (SELECT id FROM orders) candidates LIMIT 20;
SELECT id FROM (SELECT id FROM orders LIMIT 200) candidates LIMIT 20;
SELECT id FROM (SELECT id FROM orders WHERE id = $1) candidates LIMIT 20;

WITH candidates AS NOT MATERIALIZED (SELECT id FROM orders)
SELECT id FROM candidates LIMIT 20;

SELECT id FROM orders;

WITH candidates AS MATERIALIZED (SELECT id FROM orders)
SELECT id FROM candidates WHERE id = $1 LIMIT 20;

SELECT id FROM (SELECT id FROM orders) candidates WHERE id = $1 LIMIT 20;

WITH candidates AS MATERIALIZED (SELECT id FROM orders WHERE account_id = ANY($1))
SELECT id FROM candidates LIMIT 20;

WITH candidates AS MATERIALIZED (
  SELECT o.id FROM orders o JOIN accounts a ON a.id = o.account_id WHERE o.account_id = $1
) SELECT id FROM candidates LIMIT 20;

WITH candidates AS MATERIALIZED (
  SELECT o.id FROM orders o CROSS JOIN accounts a WHERE o.account_id = $1
) SELECT id FROM candidates LIMIT 20;

WITH RECURSIVE walk(id, depth) AS MATERIALIZED (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id WHERE w.depth < 5
) SELECT id FROM walk LIMIT 20;

WITH RECURSIVE walk(id, depth) AS MATERIALIZED (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1 FROM accounts a JOIN walk w ON a.id = w.id
) SELECT id FROM walk LIMIT 20;

WITH unused AS MATERIALIZED (SELECT id FROM orders)
SELECT 1;

SELECT id FROM orders WHERE id IN (SELECT id FROM accounts) LIMIT 20;
SELECT id FROM orders WHERE id IN (SELECT id FROM accounts WHERE id = $1) LIMIT 20;

SELECT (SELECT id FROM orders) LIMIT 20;
SELECT (SELECT id FROM orders LIMIT 1) LIMIT 20;
SELECT 1 WHERE EXISTS (SELECT id FROM orders) LIMIT 20;
SELECT 1 WHERE EXISTS (SELECT id FROM orders WHERE id = $1) LIMIT 20;

UPDATE orders SET account_id = (SELECT id FROM accounts) WHERE id = $1;
UPDATE orders SET account_id = (SELECT id FROM accounts LIMIT 1) WHERE id = $1;
DELETE FROM orders WHERE id = $1 AND EXISTS (SELECT id FROM accounts);
DELETE FROM orders WHERE id = $1 AND EXISTS (SELECT id FROM accounts LIMIT 1);
