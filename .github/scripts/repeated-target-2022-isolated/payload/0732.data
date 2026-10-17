-- Generic scopes keep quoted names and UTF-8 offsets independent of consumer policy.
WITH unused AS (SELECT * FROM private.ignored),
     base(id) AS (SELECT t.id FROM public.items AS t),
     used AS (SELECT b.id FROM base AS b)
SELECT a.id, "B"."Id", (SELECT s.id FROM scalar_source s WHERE s.id = a.id)
FROM public.items AS a
INNER JOIN "Schéma"."Items" AS "B" ON a.id = "B"."Id" AND a.owner = "B".owner
LEFT JOIN optional_items o ON a.id = o.id
WHERE a.id = "B"."Id"
  AND (a.owner = "B".owner OR a.owner = o.owner)
  AND NOT (a.id = o.id)
  AND CASE WHEN a.id = o.id THEN true ELSE false END
  AND (a.id = o.id) IS TRUE
  AND EXISTS (SELECT 1 FROM used u WHERE u.id = a.id)
  AND NOT EXISTS (SELECT 1 FROM other_table a WHERE a.id = 1)
GROUP BY a.id, "B"."Id"
HAVING a.id = "B"."Id"
ORDER BY (SELECT 1 FROM ordering r WHERE r.id = a.id)
LIMIT (SELECT 1 FROM limits l)
OFFSET (SELECT 0 FROM offsets f);

SELECT a.id FROM one a JOIN two b USING (id)
UNION ALL
SELECT a.id FROM three a NATURAL JOIN four b;

WITH RECURSIVE self_ref(id) AS (
 SELECT 1 UNION ALL SELECT s.id FROM self_ref s
), mutual_a AS (SELECT b.id FROM mutual_b b),
mutual_b AS (SELECT a.id FROM mutual_a a)
SELECT s.id FROM self_ref s;

WITH outer_cte AS (SELECT 1)
SELECT (WITH outer_cte AS (SELECT 2) SELECT * FROM outer_cte)
FROM outer_cte;

SELECT t.id, public.items.id, missing.id, id FROM public.items t;
SELECT a.id FROM first_table a, second_table a;
SELECT j.id, a.id FROM (one a JOIN two b ON a.id = b.id) AS j;
SELECT a.id FROM one a, (SELECT a.id) d, LATERAL (SELECT a.id) l;
SELECT a.id FROM one a RIGHT JOIN two b ON a.id = b.id FULL JOIN three c ON b.id = c.id CROSS JOIN four d;
