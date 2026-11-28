SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND ('x'::text + 'y'::text)
JOIN accounts b ON b.id = o.account_id AND ('x'::text + 'y'::text);

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND -('x'::text)
JOIN accounts b ON b.id = o.account_id AND -('x'::text);

SELECT
    (SELECT a.x BETWEEN a.lo AND a.hi FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.x BETWEEN b.lo AND b.hi FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT CAST(a.x IS NULL AS integer) BETWEEN 0 AND 1 FROM accounts a WHERE a.id = o.account_id),
    (SELECT CAST(b.x IS NULL AS integer) BETWEEN 0 AND 2 FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT lower((SELECT a.name FROM accounts a WHERE a.id = o.account_id))
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id);

SELECT
    sum((SELECT a.n FROM accounts a WHERE a.id = o.account_id)) FILTER (WHERE o.enabled),
    sum((SELECT b.n FROM accounts b WHERE b.id = o.account_id)) FILTER (WHERE NOT o.enabled)
FROM orders o;

SELECT
    sum((SELECT a.n FROM accounts a WHERE a.id = o.account_id)) FILTER (WHERE o.enabled),
    sum((SELECT b.m FROM accounts b WHERE b.id = o.account_id)) FILTER (WHERE o.enabled)
FROM orders o;

SELECT sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id) +
    (SELECT b.m FROM accounts b WHERE b.id = o.account_id)
) FILTER (WHERE o.enabled)
FROM orders o;

SELECT
    sum(o.amount) OVER w,
    avg(o.amount) OVER (ROWS (SELECT a.n FROM accounts a WHERE a.id = $1) PRECEDING)
FROM orders o
WINDOW w AS (ROWS (SELECT a.n FROM accounts a WHERE a.id = $1) PRECEDING);

SELECT
    sum(o.amount) OVER w,
    avg(o.amount) OVER (ROWS (SELECT a.n FROM accounts a WHERE a.id = $1) PRECEDING)
FROM orders o
WINDOW w AS (ROWS BETWEEN CURRENT ROW AND (SELECT a.n FROM accounts a WHERE a.id = $1) FOLLOWING);

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id
    AND 9.25 % 2.50 = 1.75::numeric
    AND 1::real + 2::real > 0::real
    AND DATE '2020-01-01' < DATE '2021-01-01'
    AND ('x'::text || 'y'::text) = 'xy'::text
    AND (NOT (a.active IS NULL) OR (a.locked IS NULL))
    AND (~(2::integer)) = (-3::integer)
    AND pg_catalog.length('xy') > 0
JOIN accounts b ON b.id = o.account_id
    AND 9.25 % 2.50 = 1.75::numeric
    AND 1::real + 2::real > 0::real
    AND DATE '2020-01-01' < DATE '2021-01-01'
    AND ('x'::text || 'y'::text) = 'xy'::text
    AND (NOT (b.active IS NULL) OR (b.locked IS NULL))
    AND (~(2::integer)) = (-3::integer)
    AND pg_catalog.length('xy') > 0;

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND 1::integer = 1::numeric
JOIN accounts b ON b.id = o.account_id AND 1::integer = 1::numeric;

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND (1 = a.status OR a.status IS NULL)
JOIN accounts b ON b.id = o.account_id AND (1 = b.status OR b.status IS NULL);

SELECT
    (SELECT a.x LIKE a.pattern FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.x ILIKE b.pattern FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT 'abc' LIKE 'a%' FROM accounts a WHERE a.id = o.account_id),
    (SELECT 'abc' ILIKE 'A%' ESCAPE '#' FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    lag(o.id, 0, (SELECT a.n FROM accounts a WHERE a.id = o.account_id)) OVER (),
    pg_catalog.lead(o.id, 0, (SELECT b.n FROM accounts b WHERE b.id = o.account_id)) OVER ()
FROM orders o;

SELECT
    lag((SELECT a.n FROM accounts a WHERE a.id = o.account_id), 0, 0) OVER (),
    lead((SELECT b.n FROM accounts b WHERE b.id = o.account_id), 0) OVER ()
FROM orders o;

SELECT first_value(
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id)
) OVER ()
FROM orders o
GROUP BY (SELECT a.n FROM accounts a WHERE a.id = o.account_id);

SELECT
    sum((SELECT a.n FROM accounts a WHERE a.id = o.account_id))
        FILTER (WHERE nextval('lookup_seq') % 2 = 0),
    max((SELECT b.n FROM accounts b WHERE b.id = o.account_id))
        FILTER (WHERE nextval('lookup_seq') % 2 = 0)
FROM orders o;

SELECT sum(
    (SELECT a.n FROM accounts a WHERE a.id = o.account_id) +
    (SELECT b.m FROM accounts b WHERE b.id = o.account_id)
) FILTER (WHERE nextval('lookup_seq') % 2 = 0)
FROM orders o;

SELECT
    (SELECT substring(a.name, 1, 1) FROM accounts a WHERE a.id = o.account_id),
    (SELECT substring(b.name, 1, 1) FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT substring('abc' FROM 1 FOR 1) FROM accounts a WHERE a.id = o.account_id),
    (SELECT substring('xyz' FOR 1) FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT (SELECT a.n FROM accounts a WHERE a.id = o.account_id) AS n
FROM orders o
GROUP BY 1
HAVING (SELECT a.n FROM accounts a WHERE a.id = o.account_id) > 0;

SELECT
    (SELECT CASE WHEN a.enabled THEN a.raw ELSE a.converted END
     FROM accounts a WHERE a.id = o.account_id),
    (SELECT CASE WHEN b.enabled THEN b.raw ELSE b.converted END
     FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT CASE WHEN a.enabled IS NOT NULL THEN 1 ELSE 0 END
     FROM accounts a WHERE a.id = o.account_id),
    (SELECT CASE 1 WHEN 1 THEN 7 END FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT CASE WHEN a.enabled IS NOT NULL THEN 1 ELSE 2.0 END
     FROM accounts a WHERE a.id = o.account_id),
    (SELECT CASE WHEN b.enabled IS NOT NULL THEN 3 ELSE 4.0 END
     FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id
    AND pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        1
         + 1) + 1) + 1) + 1) + 1)
         + 1) + 1) + 1) + 1) + 1)
         + 1) + 1) + 1) + 1) + 1)
         + 1) + 1) + 1) + 1) + 1)
         + 1) + 1) + 1) + 1) + 1) > 0
JOIN accounts b ON b.id = o.account_id
    AND pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(pg_catalog.abs(
        1
         + 1) + 1) + 1) + 1) + 1)
         + 1) + 1) + 1) + 1) + 1)
         + 1) + 1) + 1) + 1) + 1)
         + 1) + 1) + 1) + 1) + 1)
         + 1) + 1) + 1) + 1) + 1) > 0;

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND 'x'::varchar = 'x'::text
JOIN accounts b ON b.id = o.account_id AND 'x'::varchar = 'x'::text;

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND (5::smallint % 2::bigint) = 1::bigint
JOIN accounts b ON b.id = o.account_id AND (5::smallint % 2::bigint) = 1::bigint;

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id
    AND (5::smallint + 2::bigint) = 7::bigint
    AND (5::smallint % 2::smallint) = 1::smallint
    AND (1::real + 2::double precision) = 3::double precision
    AND 2147483648 + 2147483648 = 4294967296
    AND 9223372036854775808 % 2.0 = 0.0
JOIN accounts b ON b.id = o.account_id
    AND (5::smallint + 2::bigint) = 7::bigint
    AND (5::smallint % 2::smallint) = 1::smallint
    AND (1::real + 2::double precision) = 3::double precision
    AND 2147483648 + 2147483648 = 4294967296
    AND 9223372036854775808 % 2.0 = 0.0;
