SELECT count(*)
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
HAVING (SELECT a.name FROM accounts a WHERE a.id = o.account_id) IS NOT NULL;

SELECT count(*)
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
HAVING count((SELECT a.name FROM accounts a WHERE a.id = o.account_id)) > 0;

SELECT count(*)
FROM orders o
WHERE (SELECT a.name FROM accounts a WHERE a.id = o.account_id) IS NOT NULL
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id);

SELECT count(*)
FROM orders o
GROUP BY (
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
);

SELECT count(*)
FROM orders o
GROUP BY ROW(
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
);

SELECT count(*)
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
HAVING EXISTS (
    SELECT 1 FROM details d
    WHERE (SELECT a.name FROM accounts a WHERE a.id = d.account_id) IS NOT NULL
      AND (SELECT b.tier FROM accounts b WHERE b.id = d.account_id) IS NOT NULL
);

SELECT
    CASE WHEN o.enabled
        THEN (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
        ELSE 'unknown' END,
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    CASE WHEN o.enabled
        THEN (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
        ELSE 'unknown' END
FROM orders o;

SELECT CASE WHEN o.enabled
    THEN (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
    ELSE (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
END
FROM orders o;

SELECT
    CASE WHEN o.enabled THEN 'unknown'
        ELSE (SELECT a.name FROM accounts a WHERE a.id = o.account_id) END,
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT count(*)
FROM orders o
HAVING count((SELECT a.name FROM accounts a WHERE a.id = o.account_id)) > 0
   AND count((SELECT b.tier FROM accounts b WHERE b.id = o.account_id)) > 0;

SELECT
    CASE WHEN o.enabled THEN 'active'
        WHEN (SELECT a.enabled FROM accounts a WHERE a.id = o.account_id) THEN 'verified'
        ELSE 'unknown'
    END,
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
FROM orders o;
