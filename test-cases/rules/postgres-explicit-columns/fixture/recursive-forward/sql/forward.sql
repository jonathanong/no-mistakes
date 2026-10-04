-- Recursive WITH exposes all sibling aliases before any body; nonrecursive WITH stays sequential.
WITH RECURSIVE first_cte AS (SELECT id FROM accounts UNION ALL TABLE later_cte), later_cte AS (SELECT id FROM accounts) SELECT id FROM first_cte;
WITH RECURSIVE first_cte AS (SELECT id FROM accounts UNION ALL TABLE "Later_CTE"), "Later_CTE" AS (SELECT id FROM accounts) SELECT id FROM first_cte;
WITH first_cte AS (SELECT * FROM later_cte), later_cte AS (SELECT id FROM accounts) SELECT id FROM first_cte;
WITH RECURSIVE first_cte AS (SELECT * FROM public.later_cte), later_cte AS (SELECT id FROM accounts) SELECT id FROM first_cte;
WITH RECURSIVE first_cte AS (SELECT * FROM later_cte), later_cte AS (SELECT id FROM accounts) SELECT id FROM first_cte;
WITH first_cte AS (SELECT id FROM accounts), later_cte AS (SELECT * FROM first_cte) SELECT id FROM later_cte;
WITH RECURSIVE first_cte AS (SELECT (SELECT id FROM later_cte) FROM accounts), later_cte AS (SELECT id FROM accounts) SELECT * FROM first_cte;
