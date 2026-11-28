SELECT a.name, b.tier
FROM orders o
LEFT JOIN accounts a ON a.id = pg_catalog.concat(o.external_key)
LEFT JOIN accounts b ON b.id = pg_catalog.concat(o.external_key);

SELECT a.name, b.tier
FROM orders o
LEFT JOIN accounts a ON a.id = pg_catalog.concat_ws('-', o.external_key)
LEFT JOIN accounts b ON b.id = pg_catalog.concat_ws('-', o.external_key);

SELECT a.name, b.tier
FROM orders o
JOIN accounts a USING (id)
JOIN extras x USING (id)
JOIN accounts b USING (id);

SELECT a.name, b.tier
FROM accounts o
LEFT JOIN accounts a USING (id)
LEFT JOIN accounts b USING (id);

SELECT (SELECT a.id FROM accounts a WHERE a.id = o.account_id) + 1
FROM orders o
ORDER BY (SELECT a.id FROM accounts a WHERE a.id = o.account_id) + ((1));

SELECT
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    row_number() OVER (PARTITION BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id))
FROM orders o;

SELECT
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    row_number() OVER (ORDER BY (SELECT b.tier FROM accounts b WHERE b.id = o.account_id))
FROM orders o;

SELECT (
    SELECT sum(d.amount) OVER w
    FROM details d
    WINDOW w AS (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    LIMIT 1
)
FROM orders o;

SELECT
    (SELECT a.id FROM accounts a WHERE a.id = o.account_id),
    row_number() OVER (ORDER BY (SELECT a.id FROM accounts a WHERE a.id = o.account_id))
FROM orders o;

SELECT a.name, b.tier
FROM accounts o
JOIN accounts a USING (id)
JOIN extras x USING (id)
JOIN accounts b USING (id);

SELECT (
    SELECT sum(d.amount) OVER w
    FROM details d
    WINDOW w AS (ROWS BETWEEN CURRENT ROW
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING),
    unused AS (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    LIMIT 1
)
FROM orders o;

SELECT (
    SELECT sum(d.amount) OVER (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    FROM details d
    LIMIT 1
)
FROM orders o;

SELECT (
    SELECT sum(d.amount) OVER w1 + sum(d.amount) OVER w2
    FROM details d
    WINDOW w1 AS (ROWS (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING),
           w2 AS (ROWS (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING)
    LIMIT 1
)
FROM orders o;

SELECT (
    SELECT pg_catalog.row_number() OVER w
    FROM details d
    WINDOW w AS (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    LIMIT 1
)
FROM orders o;

SELECT (
    SELECT pg_catalog.row_number() OVER w + sum(d.amount) OVER w
    FROM details d
    WINDOW w AS (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    LIMIT 1
)
FROM orders o;

SELECT (
    SELECT pg_catalog.row_number() OVER (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    FROM details d
    LIMIT 1
)
FROM orders o;

SELECT (
    SELECT pg_catalog.row_number() OVER w + sum(d.amount) OVER (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    FROM details d
    WINDOW w AS (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    LIMIT 1
)
FROM orders o;

SELECT (
    SELECT pg_catalog.row_number() OVER (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    + sum(d.amount) OVER (ROWS BETWEEN
        (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
        AND (SELECT b.n FROM accounts b WHERE b.id = o.account_id) FOLLOWING)
    FROM details d
    LIMIT 1
)
FROM orders o;

SELECT (
    SELECT pg_catalog.sum(d.amount) OVER (
        ORDER BY d.id
        ROWS (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
    ) + pg_catalog.avg(d.amount) OVER (
        ORDER BY d.id ASC
        ROWS (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
    )
    FROM details d
    LIMIT 1
)
FROM orders o;

SELECT (
    SELECT pg_catalog.sum(d.amount) OVER (
        ROWS (SELECT a.n FROM accounts a WHERE a.id = o.account_id) PRECEDING
    ) + pg_catalog.avg(d.amount) OVER (
        ROWS (SELECT n FROM accounts a WHERE a.id = o.account_id) PRECEDING
    )
    FROM details d
    LIMIT 1
)
FROM orders o;
