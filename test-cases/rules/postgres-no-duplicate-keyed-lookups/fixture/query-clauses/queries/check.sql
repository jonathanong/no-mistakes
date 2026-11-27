SELECT 1 LIMIT (
    SELECT 1 FROM orders o
    JOIN accounts a ON a.id = o.account_id
    JOIN accounts b ON b.id = o.account_id LIMIT 1
);

SELECT 1 OFFSET (
    SELECT 1 FROM orders o
    JOIN accounts a ON a.id = o.account_id
    JOIN accounts b ON b.id = o.account_id LIMIT 1
);

SELECT 1 FETCH FIRST (
    SELECT 1 FROM orders o
    JOIN accounts a ON a.id = o.account_id
    JOIN accounts b ON b.id = o.account_id LIMIT 1
) ROWS ONLY;

VALUES (1), (2) ORDER BY (
    SELECT 1 FROM orders o
    JOIN accounts a ON a.id = o.account_id
    JOIN accounts b ON b.id = o.account_id LIMIT 1
);

INSERT INTO orders AS o (id) VALUES (1)
ON CONFLICT (id) DO UPDATE SET
    account_name = (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
WHERE (SELECT b.active FROM accounts b WHERE b.id = o.account_id)
RETURNING (SELECT c.tier FROM accounts c WHERE c.id = o.account_id);

INSERT INTO orders (id, account_id) VALUES (1, 2)
ON CONFLICT (id) DO UPDATE SET
    account_name = (SELECT a.name FROM accounts a WHERE a.id = excluded.account_id),
    account_tier = (SELECT b.tier FROM accounts b WHERE b.id = excluded.account_id);

SELECT row_number() OVER w, count(*) OVER w
FROM orders o
WINDOW w AS (
    PARTITION BY
        (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
        (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
);

SELECT row_number() OVER (w2)
FROM orders o
WINDOW w1 AS (
    PARTITION BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
), w2 AS (
    w1 ORDER BY (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
), unused AS (
    PARTITION BY
        (SELECT c.name FROM accounts c WHERE c.id = o.account_id),
        (SELECT d.tier FROM accounts d WHERE d.id = o.account_id)
);

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND concat(a."x,self.y") <> ''
JOIN accounts b ON b.id = o.account_id AND concat(b.x, b.y) <> '';

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND concat(a."x,self.y") <> ''
JOIN accounts b ON b.id = o.account_id AND concat(b."x,self.y") <> '';
