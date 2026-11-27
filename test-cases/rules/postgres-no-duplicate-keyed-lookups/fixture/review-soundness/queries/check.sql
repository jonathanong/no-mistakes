SELECT DISTINCT ON (
    (SELECT name FROM accounts WHERE id = o.account_id),
    (SELECT tier FROM accounts WHERE id = o.account_id)
) o.id
FROM orders o;

SELECT a.name, b.tier
FROM (orders o JOIN regions r ON r.id = o.region_id) AS src
JOIN accounts a ON a.id = src.account_id
JOIN accounts b ON b.id = src.account_id;

SELECT a.name, b.tier
FROM (orders o JOIN regions r ON r.id = o.region_id)
JOIN accounts * a ON a.id = o.account_id
JOIN accounts b ON b.id = o.account_id;

SELECT a.name, b.tier
FROM (orders o JOIN regions r ON r.id = o.region_id) AS src
JOIN orders o ON o.id = src.order_id
JOIN accounts a ON a.id = o.account_id
JOIN accounts b ON b.id = o.account_id;

SELECT a.name, b.tier
FROM orders o
JOIN accounts a TABLESAMPLE SYSTEM (0) ON a.id = o.account_id
JOIN accounts b TABLESAMPLE SYSTEM (100) ON b.id = o.account_id;

SELECT
    (SELECT name FROM accounts a TABLESAMPLE SYSTEM (0) WHERE a.id = o.account_id),
    (SELECT tier FROM accounts b TABLESAMPLE SYSTEM (100) WHERE b.id = o.account_id)
FROM orders o;

SELECT a.name, b.tier
FROM orders o
JOIN ONLY accounts a ON a.id = o.account_id
JOIN accounts b ON b.id = o.account_id;

SELECT
    (SELECT name FROM ONLY accounts a WHERE a.id = o.account_id),
    (SELECT tier FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT a.name, b.tier
FROM orders o
JOIN accounts a USING (id)
FULL JOIN extras x(id) USING (id)
JOIN accounts b USING (id);

SELECT a.name, b.tier
FROM orders o JOIN accounts a USING (id),
     extras x JOIN accounts b USING (id);

SELECT a.name, b.tier
FROM orders o JOIN accounts a USING (id),
     accounts x JOIN accounts b USING (id) JOIN accounts c USING (id);

SELECT
    (SELECT name FROM accounts a WHERE a.id = o.account_id),
    (SELECT tier FROM accounts b WHERE b.id = o.account_id FOR UPDATE SKIP LOCKED)
FROM orders o;

SELECT
    (SELECT random() FROM accounts a WHERE a.id = o.account_id),
    (SELECT random() FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT
    (SELECT pg_catalog.lower(a.name) FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
FROM orders o;

SELECT a.name, b.tier
FROM orders o
JOIN accounts a USING (id)
NATURAL FULL JOIN extras x
JOIN accounts b USING (id);

SELECT
    (SELECT (SELECT random()) FROM accounts a WHERE a.id = o.account_id),
    (SELECT (SELECT random()) FROM accounts b WHERE b.id = o.account_id)
FROM orders o;
