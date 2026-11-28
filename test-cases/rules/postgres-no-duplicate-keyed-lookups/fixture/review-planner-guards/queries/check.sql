UPDATE orders AS o
SET account_name = ''
FROM incoming AS s
WHERE s.enabled
RETURNING CASE WHEN s.enabled THEN '' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = s.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = s.account_id)
END;

UPDATE orders AS o
SET account_name = ''
FROM incoming AS s
WHERE NOT s.enabled
RETURNING CASE WHEN s.enabled THEN '' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = s.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = s.account_id)
END;

UPDATE orders AS o
SET enabled = false
WHERE o.enabled
RETURNING CASE WHEN o.enabled THEN '' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = o.account_id)
END;

SELECT pg_catalog.replace(
    NULL::text,
    (SELECT a.name FROM accounts a WHERE a.id = o.id)::text,
    (SELECT b.name FROM accounts b WHERE b.id = o.id)::text
) FROM orders o;

SELECT pg_catalog.replace(
    TEXT 'abc',
    (SELECT a.name FROM accounts a WHERE a.id = o.id)::text,
    (SELECT b.name FROM accounts b WHERE b.id = o.id)::text
) FROM orders o;

SELECT app.replace(
    NULL::text,
    (SELECT a.name FROM accounts a WHERE a.id = o.id)::text,
    (SELECT b.name FROM accounts b WHERE b.id = o.id)::text
) FROM orders o;

SELECT pg_catalog.replace(
    NULL,
    'x',
    (SELECT (SELECT a.name FROM accounts a WHERE a.id = o.id) ||
            (SELECT b.name FROM accounts b WHERE b.id = o.id))::text
) FROM orders o;

SELECT CASE WHEN o.enabled THEN 0 ELSE pg_catalog.length(pg_catalog.replace(
    NULL::text,
    pg_catalog.sum((SELECT a.n FROM accounts a WHERE a.id = o.id))::text,
    pg_catalog.sum((SELECT b.n FROM accounts b WHERE b.id = o.id))::text
)) END
FROM orders o GROUP BY o.enabled;

WITH candidate AS MATERIALIZED (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) AS x,
           (SELECT b.n FROM accounts b WHERE b.id = o.id) AS y
    FROM orders o
)
SELECT * FROM candidate LIMIT 0;

WITH candidate AS MATERIALIZED (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) AS x,
           (SELECT b.n FROM accounts b WHERE b.id = o.id) AS y
    FROM orders o
)
SELECT * FROM candidate LIMIT 1;

WITH candidate AS (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) AS x,
           (SELECT b.n FROM accounts b WHERE b.id = o.id) AS y
    FROM orders o
), changes AS (
    UPDATE orders o
    SET amount = (SELECT c.n FROM accounts c WHERE c.id = o.id) +
                 (SELECT d.n FROM accounts d WHERE d.id = o.id)
    RETURNING o.amount
)
SELECT * FROM candidate LIMIT 0;

WITH RECURSIVE candidate AS (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) AS x,
           (SELECT b.n FROM accounts b WHERE b.id = o.id) AS y
    FROM orders o
), changes AS (
    DELETE FROM archived_orders WHERE expired RETURNING id
)
SELECT * FROM candidate LIMIT 0;

SELECT DISTINCT ON (o.id)
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id)
    + (SELECT r.n FROM rates r UNION ALL SELECT r.n FROM rates r LIMIT 1)
FROM orders o
ORDER BY o.id, (SELECT a.n FROM accounts a WHERE a.id = o.account_id)
    + (SELECT n FROM rates r UNION ALL SELECT r.n FROM rates r LIMIT 1);

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.account_id)
    + (SELECT r.n FROM rates r UNION ALL SELECT r.n FROM rates r LIMIT 1)
FROM orders o
ORDER BY (SELECT a.n FROM accounts a WHERE a.id = o.account_id)
    + (SELECT r.n FROM rates r UNION ALL SELECT r.m FROM rates r LIMIT 1);

SELECT pg_catalog.sum((SELECT a.n FROM accounts a WHERE a.id = o.id))
FROM orders o
ORDER BY pg_catalog.sum(ALL (SELECT a.n FROM accounts a WHERE a.id = o.id));

SELECT pg_catalog.sum((SELECT a.n FROM accounts a WHERE a.id = o.id))
FROM orders o
ORDER BY pg_catalog.sum(DISTINCT (SELECT a.n FROM accounts a WHERE a.id = o.id));

SELECT CASE o.enabled WHEN TRUE THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id) END,
       CASE WHEN NOT o.enabled THEN
    (SELECT b.n FROM accounts b WHERE b.id = o.account_id) END
FROM orders o;

SELECT CASE o.enabled WHEN FALSE THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id) END,
       CASE WHEN NOT o.enabled THEN
    (SELECT b.n FROM accounts b WHERE b.id = o.account_id) END
FROM orders o;

SELECT CASE o.enabled WHEN TRUE THEN 0 ELSE (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
) END
FROM orders o WHERE o.enabled;

SELECT pg_catalog.corr(
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.n FROM accounts b WHERE b.id = o.account_id)
) FROM orders o WHERE FALSE;

SELECT pg_catalog.corr(
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.n FROM accounts b WHERE b.id = o.account_id)
) FROM orders o;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id LIMIT 0.4),
       (SELECT b.n FROM accounts b WHERE b.id = o.id LIMIT 0.4)
FROM orders o;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id LIMIT 0.5),
       (SELECT b.n FROM accounts b WHERE b.id = o.id LIMIT 0.5)
FROM orders o;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id LIMIT (4e-1)::bigint),
       (SELECT b.n FROM accounts b WHERE b.id = o.id LIMIT (4e-1)::bigint)
FROM orders o;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id LIMIT -0.4),
       (SELECT b.n FROM accounts b WHERE b.id = o.id LIMIT -0.4)
FROM orders o;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id
          AND pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF('seed'::text, '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')) IS NOT NULL),
       (SELECT b.n FROM accounts b WHERE b.id = o.id
          AND pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF(pg_catalog.md5(NULLIF('seed'::text, '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')), '')) IS NOT NULL)
FROM orders o;

WITH offsets AS MATERIALIZED (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id) AS n
    FROM orders o
)
SELECT 1 LIMIT 0 OFFSET (SELECT pg_catalog.max(n) FROM offsets);

WITH RECURSIVE source_rows AS MATERIALIZED (
    SELECT o.id, (SELECT a.n FROM accounts a WHERE a.id = o.id) +
                 (SELECT b.n FROM accounts b WHERE b.id = o.id) AS n
    FROM orders o
), changes AS (
    UPDATE staging s SET amount = src.n
    FROM source_rows src WHERE src.id = s.id
    RETURNING s.id
)
SELECT * FROM changes LIMIT 0;
