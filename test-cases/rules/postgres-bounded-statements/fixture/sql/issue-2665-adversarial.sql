-- An array-valued projection is not one finite candidate per source row.
WITH batch AS (SELECT $1::uuid[] AS ids) DELETE FROM accounts WHERE id = ANY(ARRAY(SELECT ids FROM batch));
WITH batch AS (SELECT $1::uuid[] AS ids) DELETE FROM accounts WHERE id = ANY((SELECT array_agg(ids) FROM batch)::uuid[]);
DELETE FROM accounts WHERE id = ANY((SELECT array_agg(id) FROM accounts LIMIT 1)::uuid[]);
-- A same-named user function supplies no built-in array cardinality proof.
WITH batch AS (SELECT id FROM accounts LIMIT $1) DELETE FROM accounts WHERE id = ANY((SELECT app.array_agg(id) FROM batch)::uuid[]);
-- A missing or nullable scalar count means LIMIT ALL, even through another CTE.
WITH empty AS (SELECT 1 WHERE false), cursor_cte AS (SELECT $1::int AS page_size FROM empty) SELECT id FROM accounts LIMIT (SELECT page_size FROM cursor_cte);
WITH cursor_cte AS (SELECT $1::int AS page_size), renamed AS (SELECT page_size FROM cursor_cte AS c(other)) SELECT id FROM accounts LIMIT (SELECT page_size FROM renamed);
-- Conjuncts may complete each alternative, but incomplete alternatives never mix.
SELECT * FROM order_lines WHERE order_id = $1 AND (line_no = $2 OR line_no = $3);
SELECT * FROM order_lines WHERE (order_id = $1 AND line_no = $2) OR (order_id = $3 AND line_no > $4);
-- Different complete keys remain valid under a shared conjunct.
SELECT id FROM accounts WHERE name = $1 AND (id = $2 OR email = $3);
-- Qualified lateral dependencies require an independently bounded preceding driver.
SELECT a.id FROM orders o CROSS JOIN LATERAL (SELECT id FROM accounts WHERE id = o.account_id LIMIT 1) latest JOIN accounts a ON a.id = latest.id WHERE o.id = $1;
SELECT a.id FROM accounts a CROSS JOIN LATERAL (SELECT a.id LIMIT 1) latest WHERE a.id = latest.id;
-- More than 32 alternatives retain only unconditional pins, with conservative fallback.
SELECT id FROM accounts WHERE id = $1 OR id = $2 OR id = $3 OR id = $4 OR id = $5 OR id = $6 OR id = $7 OR id = $8 OR id = $9 OR id = $10 OR id = $11 OR id = $12 OR id = $13 OR id = $14 OR id = $15 OR id = $16 OR id = $17 OR id = $18 OR id = $19 OR id = $20 OR id = $21 OR id = $22 OR id = $23 OR id = $24 OR id = $25 OR id = $26 OR id = $27 OR id = $28 OR id = $29 OR id = $30 OR id = $31 OR id = $32 OR id = $33;
-- Opaque and expanding projected inputs are not finite element sources.
DELETE FROM accounts WHERE id = ANY(ARRAY(SELECT get_all_ids()));
WITH batch AS (SELECT get_all_ids() AS id FROM accounts LIMIT 1) DELETE FROM accounts WHERE id = ANY(ARRAY(SELECT id FROM batch));
-- Even an unused projection SRF can erase the caller row and produce LIMIT NULL.
WITH c AS (SELECT $1::int AS n, pg_catalog.generate_series(1, 0) AS ignored) SELECT id FROM accounts LIMIT (SELECT n FROM c);
WITH c AS (SELECT $1::int AS n, app.get_all_ids() AS ignored) SELECT id FROM accounts LIMIT (SELECT n FROM c);
WITH c AS (SELECT $1::int AS n ORDER BY pg_catalog.generate_series(1, 0)) SELECT id FROM accounts LIMIT (SELECT n FROM c);
WITH c AS (SELECT $1::int AS n) SELECT * FROM (WITH c AS (SELECT NULL::int AS n) SELECT id FROM accounts LIMIT (SELECT n FROM c)) nested;
-- Group-output caps cannot bound the member rows accumulated into an array.
DELETE FROM accounts WHERE id = ANY((SELECT array_agg(id) FROM accounts HAVING true)::uuid[]);
DELETE FROM accounts WHERE id = ANY((SELECT array_agg(id) FROM accounts GROUP BY ())::uuid[]);
-- Only bounded aggregate input rows may supply a projected array alias.
WITH batch AS (SELECT id FROM accounts LIMIT $1), ids AS (SELECT array_agg(id) AS v FROM batch) DELETE FROM accounts USING ids WHERE accounts.id = ANY(ids.v);
WITH ids AS (SELECT array_agg(id) AS v FROM accounts) DELETE FROM accounts USING ids WHERE accounts.id = ANY(ids.v);
-- Cartesian DNF expansion also stops at the same conservative branch ceiling.
SELECT id FROM accounts WHERE (id = $1 OR id = $2 OR id = $3 OR id = $4 OR id = $5 OR id = $6 OR id = $7 OR id = $8) AND (email = $9 OR email = $10 OR email = $11 OR email = $12 OR email = $13 OR email = $14 OR email = $15 OR email = $16);
-- The fallback preserves a complete unconditional key.
SELECT id FROM accounts WHERE id = $1 AND (name = $2 OR name = $3 OR name = $4 OR name = $5 OR name = $6 OR name = $7 OR name = $8 OR name = $9 OR name = $10 OR name = $11 OR name = $12 OR name = $13 OR name = $14 OR name = $15 OR name = $16 OR name = $17 OR name = $18 OR name = $19 OR name = $20 OR name = $21 OR name = $22 OR name = $23 OR name = $24 OR name = $25 OR name = $26 OR name = $27 OR name = $28 OR name = $29 OR name = $30 OR name = $31 OR name = $32 OR name = $33 OR name = $34);
-- CTE declaration aliases and qualified references preserve a guaranteed caller count.
WITH cursor_cte(page_size) AS (SELECT $1::int) SELECT id FROM accounts LIMIT (SELECT c.page_size FROM cursor_cte c);
-- Nullable-side predicates do not bound the preserved driver of an outer join.
SELECT a.id FROM orders o LEFT JOIN LATERAL (SELECT id FROM accounts WHERE id = o.account_id LIMIT 1) latest ON true JOIN accounts a ON a.id = latest.id WHERE latest.id = $1;
-- A same-named local catalog relation shadows the preceding alias.
SELECT a.id FROM orders accounts CROSS JOIN LATERAL (SELECT id FROM public.accounts WHERE id = accounts.id LIMIT 1) latest JOIN accounts a ON a.id = latest.id;
-- Unknown local ownership blocks a new driver-dependency proof.
SELECT a.id FROM orders o CROSS JOIN LATERAL (SELECT id FROM unknown_local WHERE id = o.account_id LIMIT 1) latest JOIN accounts a ON a.id = latest.id WHERE o.id = $1;
-- A quoted dot remains part of one driver alias, not a schema separator.
SELECT a.id FROM orders "o.Key" CROSS JOIN LATERAL (SELECT id FROM accounts WHERE id = "o.Key".account_id LIMIT 1) latest JOIN accounts a ON a.id = latest.id WHERE "o.Key".id = $1;
-- A cast/wrapper never hides data-backed set expansion behind a query result LIMIT.
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT regexp_split_to_table(name, ',')::uuid FROM accounts LIMIT 1));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT (regexp_split_to_table(name, ','))::uuid FROM accounts LIMIT 1));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT CAST(regexp_split_to_table(name, ',') AS uuid) FROM accounts LIMIT 1));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT lower(regexp_split_to_table(name, ','))::uuid FROM accounts LIMIT 1));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT COALESCE(regexp_split_to_table(name, ','), 'x')::uuid FROM accounts LIMIT 1));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT unnest(string_to_array(name, ','))::uuid FROM accounts LIMIT 1));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT jsonb_array_elements_text(to_jsonb(name))::uuid FROM accounts LIMIT 1));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT x::uuid FROM accounts a, LATERAL regexp_split_to_table(a.name, ',') AS x LIMIT 1));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT x.id::uuid FROM accounts a, LATERAL regexp_split_to_table(a.name, ',') AS x(id) LIMIT 1));
-- Derived aliases cannot launder expansion, including a wrapper that hides an opaque projection.
WITH tokens AS (SELECT regexp_split_to_table(name, ',')::uuid AS id FROM accounts LIMIT 1) SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT id::uuid FROM tokens LIMIT 1));
WITH tokens AS (SELECT COALESCE(regexp_split_to_table(name, ','), 'x') AS id FROM accounts LIMIT 1) SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT id::uuid FROM tokens LIMIT 1));
-- Existing caller-sized SRFs remain finite; scalar casts do not validate UUID value syntax.
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT generate_series(1, $2)::text::uuid));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT unnest($1::uuid[])::uuid));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT x.id::uuid FROM unnest($1::uuid[]) AS x(id)));
WITH ids AS (SELECT $1 AS id) SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT id::uuid FROM ids));
WITH ids AS (SELECT $1::uuid AS id) SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT id::uuid FROM ids));
-- Direct limited physical-column lineage survives aliases; wildcard and scalar-subquery lineage remain conservative.
WITH ids AS (SELECT id FROM accounts LIMIT 1) SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT id::uuid FROM ids));
WITH ids AS (SELECT * FROM accounts LIMIT 1) SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT id::uuid FROM ids));
SELECT id FROM accounts WHERE id = ANY(ARRAY(SELECT (SELECT id FROM accounts LIMIT 1)::uuid));
