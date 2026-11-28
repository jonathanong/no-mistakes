SELECT a.name, b.tier
FROM orders o
LEFT JOIN accounts a ON a.id = -o.raw_key
LEFT JOIN accounts b ON b.id = -o.raw_key;

SELECT a.name, b.tier
FROM orders o
LEFT JOIN accounts a ON a.id = -CAST(o.raw_key IS NULL AS integer)
LEFT JOIN accounts b ON b.id = -CAST(o.raw_key IS NULL AS integer);

SELECT
    (SELECT -a.raw_key FROM accounts a WHERE a.id = o.account_id),
    (SELECT -b.raw_key FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT count(*)
FROM orders o
GROUP BY (SELECT a.n FROM accounts a WHERE a.id = o.account_id)
HAVING pg_catalog.abs((SELECT a.n FROM accounts a WHERE a.id = o.account_id)) > 0;

SELECT count(*)
FROM orders o
GROUP BY (SELECT a.n FROM accounts a WHERE a.id = o.account_id)
HAVING pg_catalog.min((SELECT a.n FROM accounts a WHERE a.id = o.account_id)) > 0;

SELECT (
    SELECT a.name FROM accounts a WHERE a.id = o.account_id ORDER BY a.id LIMIT 1
)
FROM orders o
ORDER BY (
    SELECT a.name FROM accounts a WHERE a.id = o.account_id ORDER BY a.id ASC NULLS LAST LIMIT 1
);

SELECT (
    SELECT a.name FROM accounts a WHERE a.id = o.account_id ORDER BY a.id DESC LIMIT 1
)
FROM orders o
ORDER BY (
    SELECT a.name FROM accounts a WHERE a.id = o.account_id ORDER BY a.id DESC NULLS FIRST LIMIT 1
);

SELECT CASE WHEN o.active
    THEN pg_catalog.first_value((SELECT a.name FROM accounts a WHERE a.id = o.account_id)) OVER ()
    ELSE pg_catalog.last_value(
        (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
        || (SELECT b.name FROM accounts b WHERE b.id = o.account_id)
    ) OVER ()
END
FROM orders o;

SELECT (
    SELECT a.name FROM accounts a WHERE a.id = o.account_id ORDER BY a.id LIMIT 1
)
FROM orders o
ORDER BY (
    SELECT a.name FROM accounts a WHERE a.id = o.account_id ORDER BY a.id NULLS FIRST LIMIT 1
);
