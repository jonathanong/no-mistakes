SELECT (SELECT name FROM accounts WHERE id = o.account_id) AS name, count(*)
FROM orders o
GROUP BY (SELECT name FROM accounts WHERE id = o.account_id);

SELECT (SELECT name FROM accounts WHERE id = o.account_id) AS name
FROM orders o
ORDER BY ((SELECT name FROM accounts WHERE id = o.account_id));

SELECT DISTINCT ON ((SELECT name FROM accounts WHERE id = o.account_id))
    (SELECT name FROM accounts WHERE id = o.account_id) AS name
FROM orders o
ORDER BY (SELECT name FROM accounts WHERE id = o.account_id);

SELECT count(*)
FROM orders o
GROUP BY (SELECT name FROM accounts WHERE id = o.account_id)
ORDER BY (SELECT name FROM accounts WHERE id = o.account_id);

SELECT (SELECT name FROM accounts WHERE id = o.account_id) AS name
FROM orders o
ORDER BY (SELECT tier FROM accounts WHERE id = o.account_id);
