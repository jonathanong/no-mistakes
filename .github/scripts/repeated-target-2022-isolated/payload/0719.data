-- These shapes distinguish syntactic facts from inferred policy guarantees.
SELECT DISTINCT ON (a.id) a.*, a.id FROM app.items a JOIN items b ON (a.id) = (b.id)
WHERE NOT EXISTS (SELECT 1 FROM child c WHERE c.id = a.id)
 OR EXISTS (SELECT 1 FROM other o WHERE o.id = a.id);
SELECT CURRENT_DATE, -a.id, +a.id, a.id IS NOT TRUE, a.id IS NOT FALSE,
 a.id IS FALSE, a.id IS UNKNOWN, a.id IS NOT UNKNOWN, a.id IS NOT NULL,
 CASE a.id WHEN 1 THEN a.id = b.id END,
 f(x => a.id), f(a.id = b.id), percentile_cont(0.5) WITHIN GROUP (ORDER BY a.id)
FROM one a JOIN two b ON a.id = b.id;
SELECT * FROM one a JOIN two b ON a.id = later.id JOIN three later ON b.id = later.id;
SELECT items.id, app.items.id FROM app.items;
SELECT * FROM (one a JOIN two b ON a.id = b.id);
SELECT * FROM LATERAL (SELECT 1) AS d(renamed);
SELECT * FROM UNNEST(ARRAY[1, 2]) AS d;
SELECT * FROM one a LEFT SEMI JOIN two b ON a.id = b.id;
SELECT * FROM one a LEFT ANTI JOIN two b ON a.id = b.id;
SELECT * FROM one a ASOF JOIN two b MATCH_CONDITION(a.id > b.id) ON a.id = b.id;
SELECT * FROM (SELECT 1) d TABLESAMPLE SYSTEM (1);
SELECT * FROM t GROUP BY ALL;
SELECT * FROM t ORDER BY ALL;
SELECT * FROM t FOR JSON AUTO;
SELECT a.id AS renamed FROM one a INNER JOIN two b ON a.id = b.id;
SELECT * FROM one a LEFT OUTER JOIN two b ON a.id = b.id;
SELECT * FROM one a RIGHT OUTER JOIN two b ON a.id = b.id;
SELECT * FROM one a SEMI JOIN two b ON a.id = b.id;
SELECT * FROM one a RIGHT SEMI JOIN two b ON a.id = b.id;
SELECT * FROM one a ANTI JOIN two b ON a.id = b.id;
SELECT * FROM one a RIGHT ANTI JOIN two b ON a.id = b.id;
SELECT * FROM one a, two b JOIN three c ON a.id = c.id;
