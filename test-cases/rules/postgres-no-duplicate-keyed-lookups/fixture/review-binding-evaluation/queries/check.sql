SELECT a.name, b.tier
FROM orders o
JOIN accounts a ON a.id = o.raw_key + 0::bigint
JOIN accounts b ON b.id = o.raw_key + 0::bigint;

SELECT a.name, b.tier
FROM orders o
JOIN accounts a ON a.id = CAST(o.raw_key IS NULL AS integer) + 0::bigint
JOIN accounts b ON b.id = CAST(o.raw_key IS NULL AS integer) + 0::bigint;

SELECT
    (SELECT a.raw_key + 0::bigint FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.raw_key + 0::bigint FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT *
FROM orders k
CROSS JOIN LATERAL (
    SELECT p.name AS old_owner, q.name AS new_owner, r.name AS current_owner
    FROM orders o
    LEFT JOIN profiles p ON p.owner_id = k.id
    JOIN accounts a USING (id) AS k
    LEFT JOIN profiles q ON q.owner_id = k.id
    LEFT JOIN profiles r ON r.owner_id = k.id
) s;

SELECT
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id AND a.active = true),
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id AND active = true)
FROM orders o;

SELECT
    (SELECT a.name FROM accounts a WHERE a.active = true),
    (SELECT b.tier FROM accounts b WHERE active = true);

SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id) IS NOT NULL
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id);

SELECT count((SELECT a.name FROM accounts a WHERE a.id = o.account_id))
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id);

SELECT CASE WHEN count(*) > 0
    THEN pg_catalog.min((SELECT a.name FROM accounts a WHERE a.id = o.account_id))
    ELSE pg_catalog.max((SELECT a.name FROM accounts a WHERE a.id = o.account_id))
END
FROM orders o;

SELECT CASE WHEN count(*) > 0
    THEN min((SELECT a.name FROM accounts a WHERE a.id = o.account_id)) FILTER (WHERE o.enabled)
    ELSE max((SELECT a.name FROM accounts a WHERE a.id = o.account_id)) FILTER (WHERE o.enabled)
END
FROM orders o;

SELECT CASE WHEN o.enabled
    THEN app.normalize((SELECT a.name FROM accounts a WHERE a.id = o.account_id))
    ELSE app.normalize((SELECT a.name FROM accounts a WHERE a.id = o.account_id))
END
FROM orders o;
