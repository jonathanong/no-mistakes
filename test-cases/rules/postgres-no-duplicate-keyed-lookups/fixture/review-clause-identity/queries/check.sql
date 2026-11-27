MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN NOT MATCHED AND
    (SELECT a.enabled FROM accounts a WHERE a.id = s.account_id)
    THEN DO NOTHING
WHEN NOT MATCHED BY TARGET AND
    (SELECT b.enabled FROM accounts b WHERE b.id = s.account_id)
    THEN DO NOTHING;

MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN MATCHED AND
    (SELECT a.enabled FROM accounts a WHERE a.id = o.account_id)
    THEN DO NOTHING
WHEN NOT MATCHED BY SOURCE AND
    (SELECT b.enabled FROM accounts b WHERE b.id = o.account_id)
    THEN DO NOTHING;

SELECT
    (SELECT a.name FROM accounts a WHERE a.owner_id = o.id ORDER BY a.id LIMIT 1),
    (SELECT b.email FROM accounts b WHERE b.owner_id = o.id ORDER BY b.id ASC NULLS LAST LIMIT 1),
    (SELECT c.name FROM accounts c WHERE c.owner_id = o.id ORDER BY c.id NULLS FIRST LIMIT 1)
FROM orders o;

SELECT
    (SELECT a.name FROM accounts a WHERE a.owner_id = o.id ORDER BY a.id DESC LIMIT 1),
    (SELECT b.email FROM accounts b WHERE b.owner_id = o.id ORDER BY b.id DESC NULLS FIRST LIMIT 1)
FROM orders o;

SELECT a.name, b.tier
FROM public.orders
CROSS JOIN orders_backup AS "public.orders"
JOIN accounts a ON a.id = "public.orders".account_id
JOIN accounts b ON b.id = "public.orders".account_id;

SELECT a.name, b.tier
FROM public.orders
CROSS JOIN orders_backup AS "public.orders"
JOIN accounts a ON a.id = orders.account_id
JOIN accounts b ON b.id = "public.orders".account_id;

SELECT array_agg(
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
    ORDER BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
)
FROM orders o;

SELECT array_agg(
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
    ORDER BY (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
)
FROM orders o;

SELECT row_number() OVER (
    PARTITION BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
    ORDER BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
)
FROM orders o;

SELECT
    row_number() OVER (PARTITION BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)),
    row_number() OVER (ORDER BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id))
FROM orders o;

SELECT
    row_number() OVER (PARTITION BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)),
    row_number() OVER (ORDER BY (SELECT b.tier FROM accounts b WHERE b.id = o.account_id))
FROM orders o;

SELECT row_number() OVER (ORDER BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id))
FROM orders o
GROUP BY o.account_id, (SELECT a.name FROM accounts a WHERE a.id = o.account_id);

SELECT
    (SELECT a.name FROM accounts a WHERE a.owner_id = o.id ORDER BY a.id USING > LIMIT 1),
    (SELECT b.email FROM accounts b WHERE b.owner_id = o.id ORDER BY b.id USING > LIMIT 1)
FROM orders o;
