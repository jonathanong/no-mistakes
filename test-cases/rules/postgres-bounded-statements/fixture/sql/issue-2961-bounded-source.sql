-- A capped CTE keeps a non-id unique key.
WITH c AS (SELECT email FROM accounts ORDER BY email LIMIT $1)
UPDATE accounts AS a SET name = 'n' FROM c WHERE a.email = c.email;

-- A one-row candidate keeps its primary key.
WITH candidate AS (SELECT id FROM orders ORDER BY id LIMIT 1)
UPDATE orders SET account_id = $2 FROM candidate WHERE orders.id = candidate.id;

-- INSERT ... SELECT ... LIMIT 1 RETURNING yields one source row.
WITH inserted AS (
  INSERT INTO accounts (id, email, name)
  SELECT $1, $2, $3
  LIMIT 1
  RETURNING id
)
UPDATE orders SET account_id = inserted.id FROM inserted WHERE orders.id = inserted.id;

-- VALUES RETURNING is one caller row.
WITH inserted AS (
  INSERT INTO accounts (id, email, name)
  VALUES ($1, $2, $3)
  RETURNING id
)
UPDATE orders SET account_id = inserted.id FROM inserted WHERE orders.id = inserted.id;

-- Caller-supplied jsonb rows are bounded like jsonb_array_elements.
UPDATE accounts SET name = x.name FROM jsonb_to_recordset($1) AS x(id uuid, name text) WHERE accounts.id = x.id;
UPDATE accounts SET name = x.name FROM jsonb_populate_recordset(NULL, $1) AS x(id uuid, name text) WHERE accounts.id = x.id;
UPDATE accounts SET name = x.name FROM jsonb_array_elements($1) AS x(value) WHERE accounts.id = x.value::uuid;
UPDATE accounts SET name = 'n' WHERE id IN (SELECT (x.id)::uuid FROM jsonb_to_recordset($1) AS x(id uuid));

-- A bounded candidate joined through another table still bounds the UPDATE target.
WITH cand AS (SELECT id FROM orders ORDER BY id LIMIT 1)
UPDATE orders SET account_id = $2
FROM cand
INNER JOIN accounts ON accounts.id = cand.id
WHERE orders.id = cand.id;

-- A derived LIMIT 1 subquery bounds the target key.
UPDATE orders SET account_id = $2
FROM (SELECT id FROM orders ORDER BY id LIMIT 1) candidate
WHERE orders.id = candidate.id;
