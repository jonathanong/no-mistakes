SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND a.flag IN (a."x,self.y")
JOIN accounts b ON b.id = o.account_id AND b.flag IN (b.x, b.y);

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND a.flag IN (a.x, a.y)
JOIN accounts b ON b.id = o.account_id AND b.flag IN (b.y, b.x);

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = o.account_id AND lower(a.id) = 1
JOIN accounts b ON b.id = o.account_id AND lower(b.id) = 1;

SELECT a.name, b.tier
FROM (orders o JOIN accounts a ON a.id = o.account_id)
JOIN accounts b ON b.id = o.account_id;

SELECT g.*
FROM orders o
CROSS JOIN LATERAL generate_series(
    (SELECT a.lo FROM ranges a WHERE a.id = o.range_id),
    (SELECT b.hi FROM ranges b WHERE b.id = o.range_id)
) AS g;

SELECT a.name, b.tier
FROM LATERAL generate_series(1, 10)
JOIN accounts a ON a.id = generate_series.generate_series
JOIN accounts b ON b.id = generate_series.generate_series;

SELECT a.name, b.tier
FROM generate_series(1, 10)
JOIN accounts a ON a.id = generate_series.generate_series
JOIN accounts b ON b.id = generate_series.generate_series;

SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
FROM orders o
ORDER BY (SELECT a.name FROM accounts a WHERE a.id = "o".account_id);

SELECT row_number() OVER (
    PARTITION BY
        (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
        (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
)
FROM orders o;

SELECT array_agg(
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
    ORDER BY (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
)
FROM orders o;

SELECT percentile_cont(
    (SELECT a.percentile FROM accounts a WHERE a.id = $1)
) WITHIN GROUP (
    ORDER BY (SELECT b.rank FROM accounts b WHERE b.id = $1)
)
FROM orders o;

SELECT (SELECT a."Name" FROM accounts a WHERE a.id = o.account_id)
FROM orders o
ORDER BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id);

SELECT o.id
FROM orders o
JOIN accounts a ON a.id = COALESCE($1, o.account_id)
JOIN accounts b ON b.id = COALESCE($1, o.account_id);
