-- Outer pins follow a named column projected from an inner derived table.
SELECT 1
FROM (
  SELECT q.id AS id, q.email AS email
  FROM (
    SELECT id, email FROM accounts
  ) q
) binding
WHERE binding.id = ANY($1) OR binding.email = ANY($2);

-- A caller-sized source is not a base table, so the pin is not copied onto it.
SELECT 1
FROM (
  SELECT x.value AS id
  FROM jsonb_array_elements($1) AS x(value)
) binding
WHERE binding.id = ANY($2);

-- The projected name is not an output of the inner query. The inner cap still bounds it.
SELECT 1
FROM (
  SELECT q.renamed AS id
  FROM (SELECT id FROM accounts ORDER BY id LIMIT 1) q
) binding
WHERE binding.id = $1;

-- The inner table already has its own alternatives, so the outer pin is not copied.
SELECT 1
FROM (
  SELECT id, email FROM accounts
  WHERE id = $1 OR email = $2
) binding
WHERE binding.id = ANY($3) OR binding.email = ANY($4);

-- A disjunction on a caller-sized item is not a table pin.
SELECT 1
FROM jsonb_array_elements($1) AS e(value)
WHERE e.value = $2 OR e.value = $3;

-- One bind element is a value equality on the primary key.
DELETE FROM accounts WHERE id = ANY(ARRAY[$1]);
