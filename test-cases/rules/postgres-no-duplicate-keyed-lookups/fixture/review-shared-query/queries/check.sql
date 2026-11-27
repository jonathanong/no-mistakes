SELECT a.name, b.tier
FROM unnest(ARRAY[1, 2]) AS u(id)
JOIN accounts a ON a.id = u.id
JOIN accounts b ON b.id = u.id;

SELECT a.name, b.tier
FROM unnest(ARRAY[1, 2])
JOIN accounts a ON a.id = unnest.unnest
JOIN accounts b ON b.id = unnest.unnest;

SELECT u.name
FROM orders o
CROSS JOIN unnest(ARRAY[
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
]) AS u(name);

SELECT q.n
FROM orders o
CROSS JOIN LATERAL (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.account_id) AS n
    LIMIT (SELECT b.n FROM accounts b WHERE b.id = o.account_id)
) q;

SELECT q.n
FROM orders o
CROSS JOIN LATERAL (
    (SELECT (SELECT a.n FROM accounts a WHERE a.id = o.account_id) AS n)
    OFFSET (SELECT b.n FROM accounts b WHERE b.id = o.account_id)
) q;

SELECT q.n
FROM orders o
CROSS JOIN LATERAL (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = o.account_id) AS n
    UNION ALL SELECT 1
    LIMIT (SELECT b.n FROM accounts b WHERE b.id = o.account_id)
) q;

SELECT
    (SELECT a.id OPERATOR(public.+) 0 FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.id OPERATOR(public.+) 0 FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT a.id::uuid FROM accounts a WHERE a.id = o.account_id),
    (SELECT CAST(b.id AS uuid) FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT a.id + 1 FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.id + 2 FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT DATE '2020-01-01' FROM accounts a WHERE a.id = o.account_id),
    (SELECT '2021-01-01'::date FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT '2020-01-01'::public.account_key FROM accounts a WHERE a.id = o.account_id),
    (SELECT '2021-01-01'::public.account_key FROM accounts b WHERE b.id = o.account_id)
FROM orders o;
