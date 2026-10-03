-- Bounded scalar leaves keep finite cardinality; nested constructors need the same leaf proof.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_id, $2]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[ARRAY[o.account_id], ARRAY[$2]]);
-- ARRAY[stored_array] is multi-dimensional: ANY flattens all stored keys, not merely one slot.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_ids]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_ids, $2]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE a.id = ANY(ARRAY[o.account_id, $2]);
UPDATE accounts a SET name = 'x' WHERE a.id = ANY(ARRAY[a.id, $1]);
-- Renamed/derived/unknown sources cannot borrow a base column's scalar type by spelling.
UPDATE accounts a SET name = 'x' FROM orders o(account_id, id) WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_id]);
UPDATE accounts a SET name = 'x' FROM (SELECT account_id FROM orders LIMIT 1) o WHERE a.id = ANY(ARRAY[o.account_id]);
UPDATE accounts a SET name = 'x' FROM missing_orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_id]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.missing_account_id]);
-- Unknown domains and intentionally malformed catalog types do not prove scalar leaves.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.domain_account_id]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.malformed_account_id]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.invalid_account_id]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.trailing_account_id]);
-- A scalar subquery/function/cast leaf needs additional type evidence; caller-only arrays still work.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[(SELECT o.account_id)]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[coalesce(o.account_id, $2)]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_id::uuid]);
DELETE FROM accounts WHERE id = ANY(ARRAY[$1, $2]);
DELETE FROM accounts WHERE id = ANY(ARRAY[]::uuid[]);
-- SQL templates use identifier placeholders; unsupported array leaves remain opaque.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_id, sql_placeholder_2]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[account_id, $2]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[coalesce((SELECT account_id FROM orders), $2)]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[coalesce(*)]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[ARRAY[(SELECT account_id FROM orders)]]);
-- Constructor casts retain source dependencies; aliases use visible catalog order.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_id, $2]::uuid[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[o.account_ids]::uuid[][]);
UPDATE accounts a SET name = 'x' FROM orders o(order_id, account_key) WHERE o.ctid = $1 AND a.id = ANY(ARRAY[o.account_key]);
UPDATE accounts a SET name = 'x' FROM orders o(order_id, unused, status, account_key) WHERE o.ctid = $1 AND a.id = ANY(ARRAY[o.account_key]);
UPDATE accounts a SET name = 'x' FROM orders o(order_id, account_key, status, account_key) WHERE o.ctid = $1 AND a.id = ANY(ARRAY[o.account_key]);
-- Catalog-declared enum leaves are scalar, while unknown domains remain opaque.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.enum_status]::text[]);
-- A positional alias named ctid shadows the physical row identifier, so it supplies no key credit.
UPDATE accounts a SET name = 'x' FROM orders o(ctid, account_key) WHERE o.ctid = $1 AND a.id = ANY(ARRAY[o.account_key]);
-- Nested constructor casts retain scalar proof; casts on stored arrays remain opaque.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[ARRAY[o.account_id]::uuid[]]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(ARRAY[ARRAY[o.account_ids]::uuid[][]]);
-- A preserved key name is still the same key, while renamed nonkeys never inherit credit.
SELECT * FROM accounts a(id) WHERE a.id = $1;
SELECT * FROM accounts a(email, id) WHERE a.id = $1;
-- Unshadowed system columns are scalar even though catalogs omit them.
SELECT 1 FROM accounts a, orders o WHERE o.id = $1 AND a.ctid = ANY(ARRAY[o.ctid]);
SELECT 1 FROM accounts a, orders o(id, account_id, status, ctid) WHERE o.id = $1 AND a.ctid = ANY(ARRAY[o.ctid]);
-- Enum matching follows schema-aware catalog names and rejects ambiguity or unknown schemas.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.enum_qualified]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.enum_foreign]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.enum_ambiguous]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.enum_missing]::text[]);
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.email = ANY(ARRAY[o.enum_bare_key]::text[]);
