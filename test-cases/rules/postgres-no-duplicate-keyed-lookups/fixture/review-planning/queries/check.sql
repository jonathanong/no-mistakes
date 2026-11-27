SELECT (
    SELECT 1
    LIMIT (SELECT a.page_size FROM accounts a WHERE a.id = o.account_id)
    OFFSET (SELECT b.page_start FROM accounts b WHERE b.id = o.account_id)
)
FROM orders o;

(SELECT o.id FROM orders o)
ORDER BY
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.email FROM accounts b WHERE b.id = o.account_id);

SELECT o.id
FROM orders o
LEFT JOIN accounts a ON a.id = o.raw_id::public.account_key
LEFT JOIN accounts b ON b.id = CAST(o.raw_id AS public.account_key);

SELECT o.id
FROM orders o
LEFT JOIN accounts a ON a.id = o.account_id AND a.enabled OPERATOR(public.+) TRUE
LEFT JOIN accounts b ON b.id = o.account_id AND b.enabled OPERATOR(public.+) TRUE;

SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id)::text
FROM orders o
ORDER BY CAST((SELECT a.name FROM accounts a WHERE a.id = o.account_id) AS text);

SELECT o.id
FROM orders o
LEFT JOIN accounts a ON a.id = o.account_id AND a.keys = '{}'::uuid[]
LEFT JOIN accounts b ON b.id = o.account_id AND b.keys = CAST('{}' AS uuid[]);

SELECT
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id LIMIT ALL),
    (SELECT c.status FROM accounts c WHERE c.id = o.account_id OFFSET 1)
FROM orders o;

SELECT
    (SELECT row_number() OVER w FROM orders i WINDOW w AS (ORDER BY i.id) LIMIT 1),
    pg_catalog.abs(o.id),
    count(*) OVER (ORDER BY o.id)
FROM orders o
WINDOW w AS (
    PARTITION BY
        (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
        (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
);

SELECT o.id
FROM orders o
LEFT JOIN accounts a ON a.id = o.opaque_key::uuid
LEFT JOIN accounts b ON b.id = CAST(o.opaque_key AS uuid);

SELECT o.id
FROM orders o
LEFT JOIN accounts a ON a.id = $1::uuid
LEFT JOIN accounts b ON b.id = CAST($1 AS uuid);

SELECT o.id
FROM orders o
LEFT JOIN accounts a ON a.id = CAST(CAST(pg_catalog.md5(CAST(o.raw_id IS NULL AS text)) AS text) AS uuid)
LEFT JOIN accounts b ON b.id = (pg_catalog.md5(CAST(o.raw_id IS NULL AS text))::text)::uuid;
