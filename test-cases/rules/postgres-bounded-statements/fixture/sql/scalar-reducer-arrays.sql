-- The reducer result is scalar; its array-valued argument still depends on the owning row.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[pg_catalog.array_to_string(o.account_ids, ',')]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[pg_catalog.concat(pg_catalog.cardinality(o.account_ids), ':')]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE a.email = ANY(ARRAY[pg_catalog.array_to_string(o.account_ids, ',')]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[app.array_to_string(o.account_ids, ',')]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[array_to_string(o.account_ids, ',')]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[pg_catalog.concat(pg_catalog.unnest(o.account_ids))]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[pg_catalog.concat(pg_catalog.array_append(o.account_ids, $2))]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(o.account_ids::text[]);
DELETE FROM accounts WHERE email = ANY(ARRAY[pg_catalog.array_to_string(ARRAY[$1, $2], ',')]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[pg_catalog.array_to_string(o.account_ids)]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[pg_catalog.array_to_string(o.account_ids, ','), o.status]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_ids]);
-- Nested finite constructor casts preserve direct-leaf proof; a stored-array leaf still has unbounded width.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[ARRAY[o.account_id, $2]::uuid[]]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[ARRAY[o.account_ids]::uuid[][]]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[ARRAY[app.lookup($2)]::uuid[]]);
