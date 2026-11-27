SELECT a.name, b.tier
FROM orders o
LEFT JOIN accounts a ON a.key = pg_catalog.lower(o.raw_key)
LEFT JOIN accounts b ON b.key = pg_catalog.lower(o.raw_key);

SELECT count(*)
FROM orders o
GROUP BY GROUPING SETS (
    ((SELECT a.name FROM accounts a WHERE a.id = o.account_id)),
    ((SELECT a.name FROM accounts a WHERE a.id = o.account_id))
);

SELECT count(*)
FROM orders o
GROUP BY GROUPING SETS (
    ((SELECT a.name FROM accounts a WHERE a.id = o.account_id)),
    ((SELECT b.tier FROM accounts b WHERE b.id = o.account_id))
);

SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
FROM orders o
ORDER BY (SELECT name FROM accounts a WHERE id = o.account_id);

SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
FROM orders o
ORDER BY (SELECT o.name FROM accounts a WHERE id = o.account_id);

SELECT DISTINCT ON (row_number() OVER w) o.id
FROM orders o
WINDOW w AS (
    PARTITION BY
        (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
        (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
), unused AS (
    PARTITION BY
        (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
        (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
);

CREATE TABLE summary AS
SELECT accounts.name, b.tier
FROM orders o
JOIN accounts ON accounts.id = o.account_id
JOIN accounts b ON b.id = o.account_id;

SELECT
    (SELECT name FROM accounts WHERE id = o.account_id AND status IS NOT NULL),
    (SELECT tier FROM accounts WHERE id = o.account_id AND status IS NOT NULL)
FROM orders o;

SELECT
    (SELECT name FROM accounts WHERE id = 2),
    (SELECT tier FROM accounts WHERE id = 2);

SELECT (VALUES (1));

INSERT INTO orders DEFAULT VALUES
RETURNING
    (SELECT a.name FROM accounts a WHERE a.id = orders.account_id),
    (SELECT b.tier FROM accounts b WHERE b.id = orders.account_id);

MERGE INTO orders o
USING incoming i ON o.id = i.id
WHEN MATCHED AND
    (SELECT a.active FROM accounts a WHERE a.id = o.account_id) AND
    (SELECT b.verified FROM accounts b WHERE b.id = o.account_id)
    THEN DELETE
WHEN NOT MATCHED THEN DO NOTHING;

TABLE accounts;

SELECT CASE WHEN o.enabled THEN 'active' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id) ||
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
END
FROM orders o;

SELECT a.name, b.tier
FROM JSON_TABLE('[1,2]'::jsonb, '$[*]' COLUMNS (id integer PATH '$')) AS u
JOIN accounts a ON a.id = u.id
JOIN accounts b ON b.id = u.id;
