SELECT
    pg_catalog.sum(o.amount) OVER (ROWS (SELECT BIGINT '1' FROM accounts a WHERE a.id = $1) PRECEDING),
    pg_catalog.avg(o.amount) OVER (ROWS (SELECT CAST('1' AS BIGINT) FROM accounts a WHERE a.id = $1) PRECEDING)
FROM orders o;

SELECT
    pg_catalog.sum(o.amount) OVER (ROWS (SELECT BIGINT '1' FROM accounts a WHERE a.id = $1) PRECEDING),
    pg_catalog.avg(o.amount) OVER (ROWS (SELECT CAST('2' AS BIGINT) FROM accounts a WHERE a.id = $1) PRECEDING)
FROM orders o;

SELECT v.name FROM orders o CROSS JOIN LATERAL (
    VALUES ((SELECT a.name FROM accounts a WHERE a.id = o.account_id)),
           ((SELECT b.name FROM accounts b WHERE b.id = o.account_id))
    LIMIT 1
) AS v(name);

SELECT v.name FROM orders o CROSS JOIN LATERAL (
    VALUES ((SELECT a.name FROM accounts a WHERE a.id = o.account_id)),
           ((SELECT b.name FROM accounts b WHERE b.id = o.account_id))
    LIMIT 2
) AS v(name);

SELECT v.name FROM orders o CROSS JOIN LATERAL (
    VALUES ((SELECT a.name FROM accounts a WHERE a.id = o.account_id)),
           ((SELECT b.name FROM accounts b WHERE b.id = o.account_id))
    ORDER BY 1 LIMIT 1
) AS v(name);

MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN MATCHED AND o.p THEN DELETE
WHEN MATCHED AND o.q THEN DELETE
RETURNING CASE WHEN o.p THEN '' WHEN o.q THEN '' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = o.account_id)
END;

MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN MATCHED AND o.p THEN DELETE
WHEN MATCHED AND o.q THEN DELETE
RETURNING (SELECT a.name FROM accounts a WHERE a.id = o.account_id) ||
          (SELECT b.name FROM accounts b WHERE b.id = o.account_id);

SELECT CASE WHEN o.enabled AND o.ready THEN
           (SELECT a.name FROM accounts a WHERE a.id = o.account_id) END,
       CASE WHEN NOT o.enabled THEN
           (SELECT b.name FROM accounts b WHERE b.id = o.account_id) END
FROM orders o;

SELECT CASE WHEN o.enabled AND o.ready THEN
           (SELECT a.name FROM accounts a WHERE a.id = o.account_id) END,
       CASE WHEN o.enabled THEN
           (SELECT b.name FROM accounts b WHERE b.id = o.account_id) END
FROM orders o;

SELECT
    CASE WHEN NOT o.enabled THEN (SELECT a.n FROM accounts a WHERE a.id = o.id) END,
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o WHERE o.enabled GROUP BY o.enabled, o.id;

SELECT
    CASE WHEN o.enabled IS NULL THEN (SELECT a.n FROM accounts a WHERE a.id = o.id) END,
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o WHERE o.enabled GROUP BY o.id, GROUPING SETS ((o.enabled), ());

SELECT
    CASE WHEN NOT o.enabled THEN (SELECT a.n FROM accounts a WHERE a.id = o.id) END,
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o WHERE NOT o.enabled GROUP BY o.enabled, o.id;

SELECT pg_catalog.lag(CASE WHEN NOT o.enabled THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
ELSE 0 END) OVER (ORDER BY o.id)
FROM orders o WHERE o.enabled;

SELECT pg_catalog.lag(CASE WHEN NOT o.enabled THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
ELSE 0 END) OVER (ORDER BY o.id)
FROM orders o WHERE NOT o.enabled;

SELECT CASE WHEN o.enabled THEN pg_catalog.lag(CASE WHEN NOT o.enabled THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
ELSE 0 END) OVER (ORDER BY o.id) END
FROM orders o;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id),
       (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o GROUP BY o.id HAVING false;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id),
       (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o GROUP BY o.id HAVING true;

SELECT
    pg_catalog.sum((SELECT a.n FROM accounts a WHERE a.id = o.id)),
    pg_catalog.sum((SELECT b.n FROM accounts b WHERE b.id = o.id))
FROM orders o GROUP BY o.id HAVING false;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.account_id) +
       (SELECT b.n FROM accounts b WHERE b.id = o.account_id)
FROM orders o WHERE false;

SELECT pg_catalog.count(o.id) +
       (SELECT a.n FROM accounts a WHERE a.id = $1) +
       (SELECT b.n FROM accounts b WHERE b.id = $1)
FROM orders o WHERE false;

SELECT pg_catalog.count(o.id) +
       (SELECT a.n FROM accounts a WHERE a.id = $1) +
       (SELECT b.n FROM accounts b WHERE b.id = $1)
FROM orders o WHERE false GROUP BY GROUPING SETS ((o.id), ());

SELECT pg_catalog.count(o.id) +
       (SELECT a.n FROM accounts a WHERE a.id = $1) +
       (SELECT b.n FROM accounts b WHERE b.id = $1)
FROM orders o WHERE false GROUP BY o.id;

SELECT pg_catalog.count(o.id) +
       (SELECT a.n FROM accounts a WHERE a.id = $1) +
       (SELECT b.n FROM accounts b WHERE b.id = $1)
FROM orders o WHERE false GROUP BY GROUPING SETS ((o.id), (o.account_id));
