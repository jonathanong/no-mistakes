-- SQL conditional forms select caller-provided arrays; ordinary calls remain opaque.
UPDATE accounts a SET name='x' FROM unnest(COALESCE($1::uuid[], ARRAY[]::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest(LEAST($1::uuid[], $2::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest(GREATEST($1::uuid[], $2::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest(NULLIF($1::uuid[], $2::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest(COALESCE(LEAST($1::uuid[], $2::uuid[]), ARRAY[]::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest(COALESCE(app.lookup($1), ARRAY[]::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest(app.coalesce($1::uuid[], ARRAY[]::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest("coalesce"($1::uuid[], ARRAY[]::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM orders o, unnest(COALESCE(o.account_ids, $1::uuid[])) u(id) WHERE o.id=$2 AND a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest(COALESCE((SELECT account_ids FROM orders), ARRAY[]::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest(COALESCE($1::app.uuid_array, ARRAY[]::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM unnest(COALESCE($1::app.uuid_value[], ARRAY[]::uuid[])) u(id) WHERE a.id=u.id;
UPDATE accounts a SET name='x' FROM generate_series(start := COALESCE($1, 1), stop := GREATEST($2, 10)) u(id) WHERE a.id=u.id;
SELECT unnest(COALESCE($1::uuid[], ARRAY[]::uuid[]));
SELECT pg_ls_dir(COALESCE($1, '.'));
SELECT app.unnest(COALESCE($1::uuid[], ARRAY[]::uuid[]));
SELECT 1 FROM string_to_table(COALESCE(DATE '2026-10-01', $1::date)::text, ',') u;
SELECT 1 FROM string_to_table(COALESCE(XML '<x/>', $1::xml)::text, ',') u;
-- The parser permits broader function metadata than PostgreSQL's conditional syntax.
SELECT 1 FROM unnest(COALESCE($1::uuid[], $2::uuid[]) OVER ()) u;
SELECT 1 FROM unnest(COALESCE($1::uuid[], $2::uuid[]) FILTER (WHERE true)) u;
SELECT 1 FROM unnest(COALESCE($1::uuid[], $2::uuid[]) WITHIN GROUP (ORDER BY $3)) u;
