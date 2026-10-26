-- DISTINCT ON keys can erase a nominally guaranteed caller row through SRFs.
WITH c AS (SELECT DISTINCT ON (pg_catalog.generate_series(1, 0)) 1 AS n) SELECT id FROM accounts LIMIT (SELECT n FROM c);
WITH c AS (SELECT DISTINCT ON (app.empty_rows()) 1 AS n) SELECT id FROM accounts LIMIT (SELECT n FROM c);
-- Ordinary DISTINCT and fixed DISTINCT ON keys retain the single caller row.
WITH c AS (SELECT DISTINCT 1 AS n) SELECT id FROM accounts LIMIT (SELECT n FROM c);
WITH c AS (SELECT DISTINCT ON (1) 1 AS n) SELECT id FROM accounts LIMIT (SELECT n FROM c);
-- A known scalar cast is one element for each bounded input row.
WITH batch AS (SELECT id FROM accounts LIMIT $1) DELETE FROM accounts WHERE id = ANY(ARRAY(SELECT id::uuid FROM batch));
-- Unsupported aggregate forms never inherit the aggregate's output-row cap.
DELETE FROM accounts WHERE id = ANY((SELECT array_agg(id) FROM accounts UNION ALL SELECT ARRAY[]::uuid[] LIMIT 1)::uuid[]);
WITH ids AS (SELECT array_agg(id) AS v FROM accounts) DELETE FROM accounts WHERE id = ANY((SELECT * FROM ids)::uuid[]);
DELETE FROM accounts WHERE id = ANY((SELECT array_agg(id ORDER BY id) FROM accounts)::uuid[]);
-- This valid inner aggregate reads an outer bare predicate, so it cannot size the target.
DELETE FROM accounts WHERE id = ANY((SELECT array_agg($1::uuid) WHERE id = $2)::uuid[]);
-- A wildcard does not invent named count provenance or table-function element metadata.
WITH seed AS (SELECT 1 AS n), c AS (SELECT * FROM seed) SELECT id FROM accounts LIMIT (SELECT n FROM c);
DELETE FROM accounts WHERE id = ANY(ARRAY(SELECT id FROM unnest($1::uuid[]) ids(id)));
-- Catalog-local qualified names stay local; unknown local scopes stay conservative.
SELECT a.id FROM public.accounts, LATERAL (SELECT id FROM accounts WHERE public.accounts.id = $1 LIMIT 1) latest JOIN accounts a ON a.id = latest.id;
SELECT a.id FROM orders o CROSS JOIN LATERAL (SELECT unknown_local.id FROM unknown_local WHERE o.account_id = $1 LIMIT 1) latest JOIN accounts a ON a.id = latest.id WHERE o.id = $2;
-- A correlated subquery retained in a CTE is still a per-row self lookup.
WITH c AS (SELECT id FROM accounts WHERE id IN (SELECT id)) SELECT id FROM c;
-- Current-database function qualification remains conservative.
DELETE FROM accounts WHERE id = ANY((SELECT postgres.pg_catalog.array_agg(id) FROM accounts)::uuid[]);
-- Temporary view dependency traversal retains correlated query reads until the view is dropped.
CREATE TEMP VIEW accounts AS SELECT id FROM public.accounts WHERE id IN (SELECT id);
SELECT id FROM accounts;
DROP VIEW accounts;
SELECT id FROM accounts;
-- Even unused named-window keys can evaluate row-erasing set-returning calls.
WITH c AS (SELECT 1 AS n WINDOW w AS (ORDER BY pg_catalog.generate_series(1, 0))) SELECT id FROM accounts LIMIT (SELECT n FROM c);
WITH c AS (SELECT 1 AS n WINDOW w AS (PARTITION BY app.empty_rows())) SELECT id FROM accounts LIMIT (SELECT n FROM c);
