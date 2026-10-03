-- Signs on numeric constants preserve a finite constructor; arbitrary operators remain unproven.
DELETE FROM accounts WHERE id = ANY(ARRAY[-1, +2, -(-3), -(+4)]);
DELETE FROM accounts WHERE id = ANY(ARRAY[-'1']);
DELETE FROM accounts WHERE id = ANY(ARRAY[~1]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[-o.account_id]);
-- pg_catalog scalar names are trusted; domains, arrays, and other namespaces remain opaque.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_0]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_1]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_2]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_3]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_4]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_5]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_6]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_7]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_8]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_9]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_10]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_11]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_12]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.network_13]::text[]);
-- Typed scalar literals retain constructor bounds; decoding stored text into arrays does not.
DELETE FROM accounts WHERE email = ANY(ARRAY[DATE '2026-01-01']::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.encoded_ids::text[]]);
DELETE FROM accounts WHERE id = ANY(ARRAY[$1::integer[]]);
DELETE FROM accounts WHERE id = ANY(ARRAY['{1,2}'::integer[]]);
-- Recovered template binds and nested constructors keep independently caller-sized arrays.
DELETE FROM accounts WHERE id = ANY(ARRAY[sql_placeholder_0::integer[]]);
DELETE FROM accounts WHERE id = ANY(ARRAY[ARRAY[1,2]::integer[]]);
DELETE FROM accounts WHERE id = ANY(ARRAY[coalesce($1, '{1,2}')::integer[]]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[coalesce(o.encoded_ids::text, '{}')::text[]]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[sql_placeholder_0::text[]]);
