-- Unknown function results can contain every stored key despite having only bind arguments.
DELETE FROM accounts WHERE id = ANY(get_ids($1));
UPDATE accounts SET name = 'changed' WHERE id = ANY(public.get_ids($1)::uuid[]);
DELETE FROM accounts WHERE id = ANY(pg_catalog.array_append($1::uuid[], $2::uuid));
DELETE FROM accounts WHERE id = ANY(pg_catalog.array_cat($1::uuid[], $2::uuid[]));
DELETE FROM accounts WHERE id = ANY(ARRAY[$1::uuid, $2::uuid]);
DELETE FROM accounts WHERE id = ANY($1::uuid[]);
DELETE FROM accounts WHERE id = ANY(pg_catalog.array_append(get_ids($1), $2));
-- The cast makes these scalar-array ANY expressions, not row-valued ANY subqueries.
DELETE FROM accounts a WHERE a.id = ANY((SELECT o.account_ids FROM orders o WHERE o.id = $1)::uuid[]);
UPDATE accounts a SET name = 'changed' WHERE a.id = ANY((SELECT o.account_ids FROM orders o)::uuid[]);
DELETE FROM accounts a WHERE a.id = ANY(SELECT o.account_id FROM orders o WHERE o.id = $1);
DELETE FROM accounts WHERE id = ANY(public.array_append($1, $2));
DELETE FROM accounts WHERE id = ANY(pg_catalog.array_append($1, $2, $3));
SELECT a.id FROM accounts a WHERE a.id = ANY((SELECT o.account_ids FROM orders o)::uuid[]);
DELETE FROM accounts WHERE id = ANY('{00000000-0000-0000-0000-000000000001}'::uuid[]);
DELETE FROM accounts WHERE id = ANY(pg_catalog.array_append(ARRAY[$1::uuid], $2::uuid));
DELETE FROM accounts WHERE id = ANY(pg_catalog.string_to_array(pg_catalog.lower($1), ','));
DELETE FROM accounts WHERE id = ANY(pg_catalog.array_append($1, public.get_id($2)));
DELETE FROM accounts WHERE id = ANY(pg_catalog.array_append($1, $2::public.custom_type));
DELETE FROM accounts WHERE id = ANY(pg_catalog.array_cat($1, $2) OVER ());
DELETE FROM accounts a WHERE a.id = ANY((SELECT o.account_ids FROM orders o WHERE o.account_id = a.id)::uuid[]);
-- A custom array-producing operator is not a caller-only builtin operation.
DELETE FROM accounts WHERE id = ANY(($1 OPERATOR(public.||) $2)::uuid[]);
DELETE FROM accounts WHERE id = ANY(pg_catalog.array_append(sql_placeholder_0, sql_placeholder_1));
