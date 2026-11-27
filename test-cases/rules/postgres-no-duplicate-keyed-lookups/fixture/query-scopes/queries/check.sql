CREATE VIEW order_accounts AS
SELECT o.id, a.name, b.tier
FROM orders o
JOIN accounts a ON a.id = o.account_id
JOIN accounts b ON b.id = o.account_id;

CREATE TABLE order_labels AS
(SELECT o.id, a.name, b.tier
 FROM orders o
 JOIN accounts a ON a.id = o.account_id
 JOIN accounts b ON b.id = o.account_id)
UNION ALL
SELECT o.id, a.name, b.tier
FROM orders o
JOIN accounts a ON a.id = o.account_id
JOIN accounts b ON b.id = o.account_id;

INSERT INTO orders AS o (account_id) VALUES ($1)
RETURNING
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    (SELECT b.tier FROM accounts b WHERE b.id = o.account_id);

DELETE FROM orders o USING pending_orders p
WHERE o.id = p.id
RETURNING
    (SELECT a.name FROM accounts a WHERE a.id = p.account_id),
    (SELECT b.tier FROM accounts b WHERE b.id = p.account_id);

SELECT d.id,
    CASE d.priority
        WHEN 1 THEN (SELECT a.name FROM accounts a WHERE a.id = d.account_id)
        WHEN 2 THEN (SELECT b.name FROM accounts b WHERE b.id = d.account_id)
        ELSE (SELECT c.name FROM accounts c WHERE c.id = d.account_id)
    END
FROM (SELECT o.id, o.priority, o.account_id FROM orders o) AS d
CROSS JOIN LATERAL (
    SELECT a.name, b.tier
    FROM teams t
    LEFT JOIN accounts a ON a.id = t.account_id
    LEFT JOIN accounts b ON b.id = t.account_id
) AS info;
