SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND a.tag IS DISTINCT FROM o.tag
JOIN accounts b ON b.id = o.account_id AND b.tag IS DISTINCT FROM o.tag;

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND a.tag IN ('x', 'y')
JOIN accounts b ON b.id = o.account_id AND b.tag IN ('x', 'y');

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND (a.tag IS NULL) IS DISTINCT FROM (o.tag IS NULL)
JOIN accounts b ON b.id = o.account_id AND (b.tag IS NULL) IS DISTINCT FROM (o.tag IS NULL);

SELECT
    (SELECT a.tag IS DISTINCT FROM o.tag FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.tag IN ('x', 'y') FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT *
FROM orders j
CROSS JOIN LATERAL (
    SELECT p.name AS old_owner, q.name AS new_owner, r.name AS current_owner
    FROM orders o
    LEFT JOIN profiles p ON p.owner_id = j.id
    CROSS JOIN JSON_TABLE('[1,2]'::jsonb, '$[*]' COLUMNS (id integer PATH '$')) AS j
    LEFT JOIN profiles q ON q.owner_id = j.id
    LEFT JOIN profiles r ON r.owner_id = j.id
) s;

SELECT
    (o.flag IS NULL, (SELECT a.name FROM accounts a WHERE a.id = o.account_id))
FROM orders o
GROUP BY o.flag, o.account_id
ORDER BY
    (NULL IS NULL, (SELECT a.name FROM accounts a WHERE a.id = o.account_id));

SELECT count(*)
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
ORDER BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id) IS NOT NULL;

SELECT count(*)
FROM orders o
GROUP BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
ORDER BY count((SELECT a.name FROM accounts a WHERE a.id = o.account_id));

SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
FROM orders o
ORDER BY (SELECT a.name FROM accounts * a WHERE a.id = o.account_id);

SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
FROM orders o
ORDER BY (SELECT a.name FROM ONLY accounts a WHERE a.id = o.account_id);

SELECT
    (SELECT ALL a.name FROM accounts a WHERE a.id = o.account_id),
    (SELECT ALL b.tier FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT (SELECT ALL a.name FROM accounts a WHERE a.id = o.account_id)
FROM orders o
ORDER BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id);

SELECT CASE WHEN count(*) > 0
    THEN array_agg((SELECT a.name FROM accounts a WHERE a.id = o.account_id) ORDER BY 1)
    ELSE array_agg((SELECT b.tier FROM accounts b WHERE b.id = o.account_id) ORDER BY 1)
END
FROM orders o;

SELECT CASE (SELECT a.enabled FROM accounts a WHERE a.id = o.account_id)
    WHEN true THEN (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
END
FROM orders o;

SELECT p.name, q.name
FROM accounts o
JOIN accounts a USING (id) AS k
LEFT JOIN profiles p ON p.account_id = o.id
LEFT JOIN profiles q ON q.account_id = k.id;

SELECT p.flag, q.flag
FROM accounts o
JOIN accounts a USING (id) AS k
FULL JOIN extras x ON x.id = o.id
LEFT JOIN profiles p ON p.flag = (o.id IS NULL)
LEFT JOIN profiles q ON q.flag = (k.id IS NULL);

SELECT p.name, q.name
FROM orders o
LEFT JOIN accounts a USING (id) AS k
LEFT JOIN profiles p ON p.account_id = k.id
LEFT JOIN profiles q ON q.account_id = k.id;

SELECT a.name, b.tier
FROM orders o
JOIN accounts a ON a.id = (CAST(o.raw_key IS NULL AS integer) + 1) + 2
JOIN accounts b ON b.id = (CAST(o.raw_key IS NULL AS integer) + 1) + 2;
