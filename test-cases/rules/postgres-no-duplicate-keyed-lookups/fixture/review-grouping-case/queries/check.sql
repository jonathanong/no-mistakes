SELECT count(*)
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
HAVING (SELECT a.name FROM accounts a WHERE a.id = o.account_id) IS NOT NULL;

SELECT count(*)
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
HAVING pg_catalog.count((SELECT a.name FROM accounts a WHERE a.id = o.account_id)) > 0;

SELECT count(*)
FROM orders o
WHERE (SELECT a.name FROM accounts a WHERE a.id = o.account_id) IS NOT NULL
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id);

SELECT count(*)
FROM orders o
GROUP BY (
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
);

SELECT count(*)
FROM orders o
GROUP BY ROW(
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
);

SELECT count(*)
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
HAVING EXISTS (
    SELECT 1 FROM details d
    WHERE (SELECT a.name FROM accounts a WHERE a.id = d.account_id) IS NOT NULL
      AND (SELECT b.tier FROM accounts b WHERE b.id = d.account_id) IS NOT NULL
);

SELECT
    CASE WHEN o.enabled
        THEN (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
        ELSE 'unknown' END,
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    CASE WHEN o.enabled
        THEN (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
        ELSE 'unknown' END
FROM orders o;

SELECT CASE WHEN o.enabled
    THEN (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
    ELSE (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
END
FROM orders o;

SELECT
    CASE WHEN o.enabled THEN 'unknown'
        ELSE (SELECT a.name FROM accounts a WHERE a.id = o.account_id) END,
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT count(*)
FROM orders o
HAVING pg_catalog.count((SELECT a.name FROM accounts a WHERE a.id = o.account_id)) > 0
   AND pg_catalog.count((SELECT b.tier FROM accounts b WHERE b.id = o.account_id)) > 0;

SELECT
    CASE WHEN o.enabled THEN 'active'
        WHEN (SELECT a.enabled FROM accounts a WHERE a.id = o.account_id) THEN 'verified'
        ELSE 'unknown'
    END,
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT COALESCE(
    0,
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.n FROM accounts b WHERE b.id = o.account_id)
)
FROM orders o;

SELECT COALESCE(
    NULL,
    $1,
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.n FROM accounts b WHERE b.id = o.account_id)
)
FROM orders o;

SELECT COALESCE(
    0,
    pg_catalog.sum((SELECT a.n FROM accounts a WHERE a.id = o.account_id)),
    pg_catalog.max((SELECT b.n FROM accounts b WHERE b.id = o.account_id))
)
FROM orders o;

SELECT CASE WHEN o.flag THEN
    pg_catalog.abs(ALL (SELECT a.n FROM accounts a WHERE a.id = o.account_id))
ELSE
    pg_catalog.abs(ALL (SELECT b.n FROM accounts b WHERE b.id = o.account_id))
END
FROM orders o;

SELECT *
FROM orders x
CROSS JOIN LATERAL (
    SELECT p.name AS p_name, q.name AS q_name
    FROM orders o
    LEFT JOIN profiles p ON p.owner_id = x.id
    CROSS JOIN XMLTABLE('/r' PASSING '<r id="2"/>'::xml COLUMNS id int PATH '@id') AS x
    LEFT JOIN profiles q ON q.owner_id = x.id
) AS s;

SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
FROM orders o
GROUP BY 1
ORDER BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id) IS NOT NULL;

SELECT (SELECT a.label FROM flags a WHERE a.id = (o.id IS NULL))
FROM orders o
JOIN flags f ON f.id = (o.id IS NULL)
GROUP BY ROLLUP(o.id)
HAVING GROUPING(o.id) = 1;

SELECT
    (SELECT a.label FROM flags a WHERE a.id = (o.id IS NULL)),
    pg_catalog.max((SELECT b.label FROM flags b WHERE b.id = (o.id IS NULL))),
    (SELECT c.label FROM flags c WHERE c.id = (o.id IS NULL))
FROM orders o
JOIN flags f ON f.id = (o.id IS NULL)
GROUP BY CUBE(o.id);

SELECT (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.account_id) AS probe
    GROUP BY probe
    HAVING (SELECT a.n FROM accounts a WHERE a.id = o.account_id) IS NOT NULL
)
FROM orders o;

SELECT (SELECT a.id FROM accounts a WHERE a.id = o.account_id)::int
FROM orders o
ORDER BY (SELECT a.id FROM accounts a WHERE a.id = o.account_id)::integer;

SELECT (SELECT a.id FROM accounts a WHERE a.id = o.account_id)::int
FROM orders o
ORDER BY (SELECT a.id FROM accounts a WHERE a.id = o.account_id)::bigint;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.account_id) AS account_id
FROM orders o
GROUP BY account_id
HAVING (SELECT a.n FROM accounts a WHERE a.id = o.account_id) IS NOT NULL;
