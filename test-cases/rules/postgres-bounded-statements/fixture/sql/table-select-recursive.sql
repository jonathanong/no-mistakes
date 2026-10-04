-- A TABLE arm naming a recursive CTE is not a physical relation read.
WITH RECURSIVE later_cte AS (SELECT id FROM safe)
SELECT id FROM safe UNION ALL TABLE later_cte;
