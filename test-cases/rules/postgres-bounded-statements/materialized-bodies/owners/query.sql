WITH pool AS MATERIALIZED (SELECT id FROM accounts)
UPDATE orders SET account_id = $1 WHERE id = $2;
WITH pool AS MATERIALIZED (SELECT id FROM accounts LIMIT 1)
UPDATE orders SET account_id = $1 WHERE id = $2;
UPDATE orders SET account_id = (SELECT id FROM accounts) WHERE id = $1;
UPDATE orders SET account_id = (SELECT id FROM accounts LIMIT 1) WHERE id = $1;
WITH changed AS (
  UPDATE orders SET account_id = $1 WHERE id = $2
  AND id IN (SELECT id FROM accounts) RETURNING id
) SELECT id FROM changed LIMIT 1;
WITH changed AS (
  UPDATE orders SET account_id = $1 WHERE id = $2
  AND id IN (SELECT id FROM accounts LIMIT 1) RETURNING id
) SELECT id FROM changed LIMIT 1;

WITH pool AS MATERIALIZED (SELECT id FROM accounts)
DELETE FROM orders WHERE id = $1;
WITH pool AS MATERIALIZED (SELECT id FROM accounts LIMIT 1)
DELETE FROM orders WHERE id = $1;
DELETE FROM orders WHERE id = $1 AND id IN (SELECT id FROM accounts);
DELETE FROM orders WHERE id = $1 AND id IN (SELECT id FROM accounts LIMIT 1);
WITH changed AS (
  DELETE FROM orders WHERE id = $1
  AND id IN (SELECT id FROM accounts) RETURNING id
) SELECT id FROM changed LIMIT 1;
WITH changed AS (
  DELETE FROM orders WHERE id = $1
  AND id IN (SELECT id FROM accounts LIMIT 1) RETURNING id
) SELECT id FROM changed LIMIT 1;

WITH pool AS MATERIALIZED (SELECT id FROM accounts)
SELECT id FROM pool LIMIT 1;

COPY (SELECT id FROM (SELECT id FROM accounts LIMIT 1) bounded LIMIT 1) TO STDOUT;
