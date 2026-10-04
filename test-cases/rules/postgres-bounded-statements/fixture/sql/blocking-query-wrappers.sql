-- An outer row cap cannot hide the complete inputs of a blocking set operation.
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) d LIMIT 1;
WITH q AS (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) SELECT * FROM q LIMIT 1;
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 INTERSECT SELECT account_id FROM orders) d LIMIT 1;
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 UNION SELECT account_id FROM orders) d LIMIT 1;
(SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) d) UNION ALL (SELECT id FROM invoices) LIMIT 1;
SELECT a.id FROM accounts a CROSS JOIN (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) d LIMIT 1;
SELECT * FROM (SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) d LIMIT 1) outer_q LIMIT 1;
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) d LIMIT 0;
WITH q AS ((SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) LIMIT 0) SELECT * FROM q LIMIT 1;
SELECT * FROM (SELECT id FROM accounts UNION ALL SELECT account_id FROM orders) d LIMIT 1;
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders WHERE id = $2) d LIMIT 1;
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT (SELECT account_id FROM orders LIMIT 1)) d LIMIT 1;
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) d WHERE false LIMIT 1;
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) d FETCH FIRST 0 ROWS ONLY;
-- The same CTE read can be reached on both streaming and blocking paths.
WITH src AS (SELECT account_id FROM orders), blocked AS (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM src), mixed AS (SELECT s.account_id FROM src s CROSS JOIN blocked b) SELECT * FROM mixed LIMIT 1;
CREATE TEMP TABLE orders (id uuid, account_id uuid);
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) d LIMIT 1;
DROP TABLE orders;
SELECT * FROM (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) d LIMIT 1;
