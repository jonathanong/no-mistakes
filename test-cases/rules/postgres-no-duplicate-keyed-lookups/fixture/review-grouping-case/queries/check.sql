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

SELECT
    COALESCE(0, (SELECT a.n FROM accounts a WHERE a.id = o.id)),
    0,
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o
GROUP BY o.id, 2;

SELECT COALESCE(0, (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id)
         + (SELECT b.n FROM accounts b WHERE b.id = o.id)
))
FROM orders o;

SELECT COALESCE(0, pg_catalog.sum((
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id)
         + (SELECT b.n FROM accounts b WHERE b.id = o.id)
)))
FROM orders o;

SELECT COALESCE((SELECT a.n FROM accounts a WHERE a.id = o.id), 0)
FROM orders o
ORDER BY "coalesce"((SELECT a.n FROM accounts a WHERE a.id = o.id), 0);

SELECT a.name, b.email
FROM XMLTABLE('/r' PASSING '<r id="1"/>'::xml COLUMNS id integer PATH '@id')
LEFT JOIN accounts a ON a.id = xmltable.id
LEFT JOIN accounts b ON b.id = xmltable.id;

SELECT a.name, b.email
FROM JSON_TABLE('[1,2]'::jsonb, '$[*]' COLUMNS (id integer PATH '$'))
LEFT JOIN accounts a ON a.id = json_table.id
LEFT JOIN accounts b ON b.id = json_table.id;

SELECT x.*
FROM orders o
CROSS JOIN XMLTABLE(
    (SELECT a.path FROM documents a WHERE a.id = o.document_id)
    PASSING (SELECT b.doc FROM documents b WHERE b.id = o.document_id)
    COLUMNS id integer PATH '@id'
) AS x;

SELECT
    COALESCE(0, (SELECT a.n FROM accounts a WHERE a.id = o.id)),
    0,
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o
GROUP BY GROUPING SETS ((o.id, 2), (o.id));

SELECT
    CASE WHEN FALSE THEN
        (SELECT a.n FROM accounts a WHERE a.id = o.id)
    ELSE 0 END,
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o;

SELECT CASE WHEN FALSE THEN (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id)
         + (SELECT b.n FROM accounts b WHERE b.id = o.id)
) ELSE 0 END
FROM orders o;

SELECT
    CASE FALSE WHEN FALSE THEN
        (SELECT a.n FROM accounts a WHERE a.id = o.id)
    ELSE 0 END,
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o;

SELECT CASE WHEN FALSE THEN pg_catalog.sum((
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id)
         + (SELECT b.n FROM accounts b WHERE b.id = o.id)
)) ELSE 0 END
FROM orders o;

SELECT CASE
    WHEN NULL THEN 0
    WHEN o.enabled THEN (
        SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id)
             + (SELECT b.n FROM accounts b WHERE b.id = o.id)
    )
    ELSE 0
END
FROM orders o;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.coalesce)
FROM orders o
ORDER BY (SELECT a.n FROM accounts a WHERE a.id = o."coalesce");

SELECT CASE
    WHEN FALSE THEN 0
    WHEN TRUE THEN 0
    ELSE (
        SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id)
             + (SELECT b.n FROM accounts b WHERE b.id = o.id)
    )
END
FROM orders o;

SELECT CASE WHEN TRUE THEN 0
    WHEN o.enabled THEN (SELECT a.n FROM accounts a WHERE a.id = o.id)
    ELSE (SELECT b.n FROM accounts b WHERE b.id = o.id)
END
FROM orders o;

SELECT CASE WHEN TRUE THEN 0 ELSE pg_catalog.sum((
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id)
         + (SELECT b.n FROM accounts b WHERE b.id = o.id)
)) END
FROM orders o;

SELECT pg_catalog.percentile_cont(
    (SELECT a.fraction FROM accounts a WHERE a.id = o.account_id)
) WITHIN GROUP (ORDER BY o.amount)
FROM orders o
GROUP BY (SELECT a.fraction FROM accounts a WHERE a.id = o.account_id);

SELECT pg_catalog.percentile_cont(0.5) WITHIN GROUP (ORDER BY
    (SELECT a.amount FROM accounts a WHERE a.id = o.account_id)
)
FROM orders o
GROUP BY (SELECT a.amount FROM accounts a WHERE a.id = o.account_id);

SELECT x.*
FROM orders o
CROSS JOIN XMLTABLE(
    XMLNAMESPACES(
        (SELECT a.uri FROM documents a WHERE a.id = o.document_id) AS p,
        (SELECT b.uri FROM documents b WHERE b.id = o.document_id) AS q
    ),
    '/r' PASSING '<r/>'::xml
    COLUMNS id integer PATH '@id'
) AS x;

SELECT pg_catalog.corr(
    (SELECT a.n FROM accounts a WHERE a.id = o.id),
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) FILTER (WHERE FALSE)
FROM orders o;

SELECT
    (SELECT a.n FROM accounts a WHERE a.id = o.id LIMIT 0),
    (SELECT b.n FROM accounts b WHERE b.id = o.id LIMIT (0)),
    (SELECT c.n FROM accounts c WHERE c.id = o.id LIMIT 1),
    (SELECT d.n FROM accounts d WHERE d.id = o.id LIMIT 1)
FROM orders o;

SELECT pg_catalog.sum(pg_catalog.sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id)
    + (SELECT b.m FROM accounts b WHERE b.id = o.account_id)
)) FILTER (WHERE FALSE) OVER ()
FROM orders o;

SELECT
    pg_catalog.sum((SELECT a.n FROM accounts a WHERE a.id = $1))
        FILTER (WHERE o.enabled) OVER (),
    pg_catalog.sum((SELECT b.m FROM accounts b WHERE b.id = $1))
        FILTER (WHERE o.enabled) OVER ()
FROM orders o
GROUP BY o.enabled;

SELECT
    pg_catalog.sum((SELECT a.n FROM accounts a WHERE a.id = $1))
        FILTER (WHERE o.enabled) OVER (),
    pg_catalog.sum((SELECT b.m FROM accounts b WHERE b.id = $1))
        FILTER (WHERE o.active) OVER ()
FROM orders o
GROUP BY o.enabled, o.active;
