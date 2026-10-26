-- The nonrecursive CTE body reads the physical accounts table despite sharing its name.
WITH accounts AS (SELECT 1 FROM accounts WHERE id::text = $1) SELECT 1 FROM accounts;
WITH earlier AS (SELECT id FROM text_ids), accounts AS (SELECT * FROM earlier) SELECT 1 FROM accounts WHERE id::text = $1;
WITH RECURSIVE accounts AS (SELECT id FROM text_ids UNION ALL SELECT id FROM accounts) SELECT 1 FROM accounts WHERE id::text = $1;
WITH accounts AS (SELECT id FROM text_ids) SELECT 1 FROM public.accounts WHERE id::text = $1;
WITH outer_name AS (SELECT 1 FROM accounts WHERE CAST(id AS text) = $1) SELECT * FROM outer_name;
SELECT 1 FROM (SELECT 1 FROM accounts WHERE id::text = $1) derived WHERE derived.id::text = $1;
SELECT 1 FROM accounts WHERE id::text = $1 UNION ALL SELECT 1 FROM text_ids WHERE id::text = $1;
