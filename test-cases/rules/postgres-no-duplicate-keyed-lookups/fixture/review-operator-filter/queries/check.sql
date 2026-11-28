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
