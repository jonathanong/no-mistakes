-- An uncapped INSERT source does not bound the later statement.
WITH inserted AS (
  INSERT INTO accounts (id, email, name)
  SELECT id, email, name FROM accounts
  RETURNING id
)
UPDATE orders SET account_id = inserted.id FROM inserted WHERE orders.id = inserted.id;

-- A nested constructor is not a single scalar.
WITH c AS (SELECT id FROM orders ORDER BY id LIMIT 1)
UPDATE orders SET account_id = $2 FROM c WHERE orders.id = ANY(ARRAY[ARRAY[c.id]]);

-- A multi-element constructor is not equality.
WITH c AS (SELECT id FROM orders ORDER BY id LIMIT 1)
UPDATE orders SET account_id = $2 FROM c WHERE orders.id = ANY(ARRAY[c.id, c.id]);

-- A wildcard CTE does not prove who owns a bare column.
WITH one AS (SELECT * FROM accounts LIMIT 1)
UPDATE orders SET account_id = $2 FROM one WHERE id = $1;

-- A constant arm projection does not receive the outer pin.
SELECT 1
FROM (
  SELECT 1 AS id, email FROM accounts
  UNION ALL
  SELECT id, email FROM accounts
) binding
WHERE binding.id = ANY($1);

-- The finding stays on the UPDATE target.
WITH cand AS (SELECT email FROM accounts ORDER BY email LIMIT 1)
UPDATE orders SET account_id = $2
FROM cand
INNER JOIN accounts ON accounts.email = cand.email
WHERE orders.account_id = accounts.id;

-- A positional alias on a base table hides its catalog columns, so bare id is not proven.
UPDATE orders SET account_id = $2
FROM accounts a(id, email, name)
WHERE id = $1;
