MERGE INTO orders o USING staged_orders s ON s.id = o.id
WHEN MATCHED AND o.enabled THEN DELETE
RETURNING CASE WHEN old.enabled THEN 0 ELSE
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
END;

MERGE INTO orders o USING staged_orders s ON s.id = o.id
WHEN MATCHED AND o.enabled THEN DELETE
WHEN MATCHED AND o.archived THEN DELETE
RETURNING CASE WHEN old.enabled THEN 0 ELSE
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
END;

MERGE INTO orders o USING staged_orders s ON s.id = o.id
WHEN MATCHED AND o.enabled THEN DELETE
WHEN MATCHED THEN UPDATE SET enabled = false
RETURNING CASE WHEN o.enabled THEN 0 ELSE
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
END;

MERGE INTO orders o USING staged_orders s ON s.id = o.id
WHEN MATCHED AND o.enabled THEN UPDATE SET enabled = false
WHEN MATCHED THEN DELETE
RETURNING (SELECT a.n FROM accounts a WHERE a.id = o.id) +
          (SELECT b.n FROM accounts b WHERE b.id = o.id);

SELECT *
FROM orders o
JOIN accounts a ON a.id = '1'::int
JOIN accounts b ON b.id = CAST('1' AS integer);

SELECT *
FROM orders o
JOIN accounts a ON a.id = '1'::smallint
JOIN accounts b ON b.id = CAST('1' AS bigint);

SELECT pg_catalog.sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) FILTER (WHERE (SELECT c.valid FROM accounts c WHERE c.id = o.id))
OVER (ORDER BY o.id ROWS BETWEEN 2 FOLLOWING AND 1 FOLLOWING)
FROM orders o;

SELECT pg_catalog.sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) OVER (ORDER BY o.id ROWS BETWEEN 2 PRECEDING AND 1 PRECEDING)
FROM orders o;

SELECT pg_catalog.sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) OVER (ORDER BY o.id ROWS BETWEEN 1 PRECEDING AND CURRENT ROW)
FROM orders o;

SELECT pg_catalog.sum(pg_catalog.sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
)) OVER (ROWS BETWEEN 2 FOLLOWING AND 1 FOLLOWING)
FROM orders o;

SELECT pg_catalog.sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) OVER (ORDER BY o.id ROWS BETWEEN 2 FOLLOWING AND $1 FOLLOWING)
FROM orders o;

SELECT x.*
FROM orders o
CROSS JOIN XMLTABLE(
    XMLNAMESPACES(
        (SELECT a.uri FROM documents a WHERE a.id = o.document_id) AS p,
        (SELECT b.uri FROM documents b WHERE b.id = o.document_id) AS q
    ),
    '/r' PASSING NULL::xml
    COLUMNS id integer PATH '@id'
) AS x;

SELECT x.*
FROM orders o
CROSS JOIN XMLTABLE(
    XMLNAMESPACES(
        (SELECT a.uri FROM documents a WHERE a.id = o.document_id) AS p,
        (SELECT b.uri FROM documents b WHERE b.id = o.document_id) AS q
    ),
    '/r' PASSING o.document
    COLUMNS id integer PATH '@id'
) AS x;

SELECT x.*
FROM orders o
CROSS JOIN XMLTABLE(
    (SELECT '/r' FROM documents a WHERE a.id = o.document_id)
    PASSING (SELECT b.document FROM documents b WHERE b.id = o.document_id)
    COLUMNS id integer PATH '@id'
) AS x;

SELECT pg_catalog.sum(CASE WHEN NOT o.enabled THEN (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
) ELSE 0 END) FILTER (WHERE o.enabled)
FROM orders o;

SELECT pg_catalog.sum(CASE WHEN NOT o.enabled THEN (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
) ELSE 0 END) FILTER (WHERE NOT o.enabled)
FROM orders o;

SELECT CASE WHEN NOT o.enabled THEN (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
) END
FROM orders o WHERE o.enabled;

SELECT CASE WHEN NOT o.enabled THEN (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
) END
FROM orders o WHERE NOT o.enabled;

SELECT
    pg_catalog.sum(o.amount) OVER (ROWS (SELECT '1'::bigint FROM accounts a WHERE a.id = $1) PRECEDING),
    pg_catalog.avg(o.amount) OVER (ROWS (SELECT E'1'::bigint FROM accounts a WHERE a.id = $1) PRECEDING)
FROM orders o;

SELECT
    pg_catalog.sum(o.amount) OVER (ROWS (SELECT '1'::bigint FROM accounts a WHERE a.id = $1) PRECEDING),
    pg_catalog.avg(o.amount) OVER (ROWS (SELECT E'2'::bigint FROM accounts a WHERE a.id = $1) PRECEDING)
FROM orders o;

SELECT CASE 1 WHEN NULL THEN (
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) WHEN 2 THEN (
    (SELECT c.n FROM accounts c WHERE c.id = o.id) +
    (SELECT d.n FROM accounts d WHERE d.id = o.id)
) WHEN 1 THEN (
    (SELECT e.n FROM accounts e WHERE e.id = o.id) +
    (SELECT f.n FROM accounts f WHERE f.id = o.id)
) END
FROM orders o;

SELECT (SELECT pg_catalog.sum(o.n)),
       (SELECT a.n FROM accounts a WHERE a.id = $1),
       (SELECT b.n FROM accounts b WHERE b.id = $1)
FROM orders o WHERE FALSE;

SELECT (SELECT pg_catalog.abs(o.n)),
       (SELECT a.n FROM accounts a WHERE a.id = $1),
       (SELECT b.n FROM accounts b WHERE b.id = $1)
FROM orders o WHERE FALSE;

SELECT (SELECT pg_catalog.row_number() OVER (ORDER BY o.n)),
       (SELECT a.n FROM accounts a WHERE a.id = $1),
       (SELECT b.n FROM accounts b WHERE b.id = $1)
FROM orders o WHERE FALSE;

SELECT pg_catalog.sum(CASE WHEN NOT o.enabled THEN (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
) ELSE 0 END) FILTER (WHERE o.enabled) OVER ()
FROM orders o;

SELECT pg_catalog.sum(CASE WHEN NOT o.enabled THEN (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
) ELSE 0 END) FILTER (WHERE NOT o.enabled) OVER ()
FROM orders o;

SELECT pg_catalog.sum(pg_catalog.sum((
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
))) FILTER (WHERE FALSE) OVER ()
FROM orders o;

MERGE INTO orders o USING staged_orders s ON s.id = o.id
WHEN MATCHED AND o.enabled THEN DELETE
WHEN NOT MATCHED BY SOURCE AND o.archived THEN DELETE
RETURNING CASE WHEN old.enabled THEN 0 ELSE
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
END;

SELECT x.*
FROM orders o
CROSS JOIN XMLTABLE(
    XMLNAMESPACES(
        (SELECT a.uri FROM documents a WHERE a.id = o.document_id) AS p,
        (SELECT b.uri FROM documents b WHERE b.id = o.document_id) AS q
    ),
    '/r' PASSING NULL
    COLUMNS id integer PATH '@id'
) AS x;

SELECT pg_catalog.sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) OVER (ORDER BY o.id ROWS UNBOUNDED PRECEDING)
FROM orders o;

SELECT pg_catalog.sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) OVER (ORDER BY o.id ROWS 1 PRECEDING)
FROM orders o;

SELECT pg_catalog.lag((
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
)) OVER (ORDER BY o.id ROWS BETWEEN 2 FOLLOWING AND 1 FOLLOWING)
FROM orders o;

SELECT pg_catalog.sum(pg_catalog.sum((
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) +
           (SELECT b.n FROM accounts b WHERE b.id = o.id)
))) FILTER (WHERE o.enabled) OVER ()
FROM orders o GROUP BY o.enabled;

SELECT CASE 1 WHEN 1.0 THEN (
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) ELSE 0 END
FROM orders o;

SELECT pg_catalog.sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
) OVER (ORDER BY o.id RANGE UNBOUNDED PRECEDING)
FROM orders o;

SELECT
    (SELECT pg_catalog.lower(o.name) || 'x'::text FROM accounts a WHERE a.id = o.id),
    (SELECT pg_catalog.lower(o.name) || 'y'::text FROM accounts b WHERE b.id = o.id)
FROM orders o;

SELECT
    (SELECT pg_catalog.substr('abc'::text, 1, 2) || 'x'::text FROM accounts a WHERE a.id = o.id),
    (SELECT pg_catalog.substr('abc'::text, 1, 2) || 'y'::text FROM accounts b WHERE b.id = o.id)
FROM orders o;

SELECT
    (SELECT pg_catalog.abs(o.amount) + 2 FROM accounts a WHERE a.id = o.id),
    (SELECT pg_catalog.abs(o.amount) + 3 FROM accounts b WHERE b.id = o.id)
FROM orders o;
