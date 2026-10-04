-- Wrappers change argument shape, not a trusted scalar reducer's one-value result.
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids::uuid[], ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids[1:2], ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids::app.uuid_array, ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(COALESCE(o.account_ids::uuid[], $2::uuid[]), ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids::uuid[], ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(app.lookup(o.account_ids)::uuid[], ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.concat(pg_catalog.unnest(o.account_ids)::text)]);
-- Direct final leaves remain strict: casts, slices and domains do not prove finite width.
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.account_ids::uuid[]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.account_ids[1:2]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[o.account_ids::app.uuid_array]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.id=ANY(ARRAY[COALESCE(o.account_ids, $2::uuid[])]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids[:$2], ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids[$2:], ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids[:], ',')]);
UPDATE accounts a SET name='x' FROM orders o, profiles p WHERE o.id=$1 AND p.account_id=$2 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids[pg_catalog.length(p.bio):], ',')]);
UPDATE accounts a SET name='x' FROM orders o, profiles p WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids[pg_catalog.length(p.bio):], ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(o.account_ids[app.limit($2):], ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string((pg_catalog.array_append(o.account_ids, $2))[1:2], ',')]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[pg_catalog.array_to_string(x.account_ids[1:2], ',')]);
