SELECT a.name, b.name, c.name
FROM orders o
LEFT JOIN flags a ON a.flag = (o.id IS NULL)
FULL JOIN extras x ON false
LEFT JOIN flags b ON b.flag = (o.id IS NULL)
LEFT JOIN flags c ON c.flag = (o.id IS NULL);

SELECT a.name, b.name, c.name
FROM orders o
LEFT JOIN flags a ON a.flag = (o.id IS NULL)
RIGHT JOIN extras x ON false
LEFT JOIN flags b ON b.flag = (o.id IS NULL)
LEFT JOIN flags c ON c.flag = (o.id IS NULL);

SELECT q.*
FROM orders o
CROSS JOIN LATERAL (
    SELECT a.name AS first_flag, b.name AS second_flag
    FROM teams t
    LEFT JOIN flags a ON a.flag = (o.id IS NULL)
    FULL JOIN extras x ON x.id = t.id
    LEFT JOIN flags b ON b.flag = (o.id IS NULL)
) q;

SELECT a.name, b.name
FROM accounts o
JOIN accounts a USING (id) AS first_key
JOIN accounts b USING (id) AS second_key;

SELECT a.name, b.name
FROM accounts o
LEFT JOIN accounts a USING (id) AS first_key
FULL JOIN extras x ON false
LEFT JOIN accounts b USING (id) AS second_key;

SELECT (SELECT a.name FROM accounts AS a WHERE a.id = o.account_id)
FROM orders o
ORDER BY (SELECT a.name FROM accounts a WHERE a.id = o.account_id);

SELECT
    (SELECT a.name FROM accounts AS a WHERE a.id = o.account_id),
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id)
FROM orders o;

SELECT
    sum(o.amount) OVER (ROWS (SELECT a.n FROM accounts AS a WHERE a.id = $1) PRECEDING),
    avg(o.amount) OVER (ROWS (SELECT a.n FROM accounts a WHERE a.id = $1) PRECEDING)
FROM orders o;

SELECT
    sum(o.amount) OVER (ROWS (SELECT a.n FROM accounts a WHERE a.id = $1) PRECEDING),
    avg(o.amount) OVER (ROWS BETWEEN CURRENT ROW AND
        (SELECT a.n FROM accounts a WHERE a.id = $1) FOLLOWING)
FROM orders o;

SELECT
    (SELECT b.name FROM flags b WHERE b.flag = (x.id IS NULL)),
    (SELECT c.name FROM flags c WHERE c.flag = (x.id IS NULL))
FROM orders o
LEFT JOIN extras x ON
    (SELECT a.enabled FROM flags a WHERE a.flag = (x.id IS NULL));

SELECT 1
FROM orders o
CROSS JOIN teams t
CROSS JOIN LATERAL (
    SELECT 1
    FROM teams t JOIN teams u USING (id) AS o,
         (VALUES (1)) v(n)
         JOIN accounts a ON a.id = o.id
         JOIN accounts b ON b.id = t.id
) q;

SELECT 1
FROM orders o
CROSS JOIN teams t
CROSS JOIN LATERAL (
    SELECT 1
    FROM teams t JOIN teams u USING (id) AS o,
         (VALUES (1)) v(n)
         JOIN accounts a ON a.id = o.id
         JOIN accounts b ON b.id = o.id
) q;

SELECT 1
FROM orders o
CROSS JOIN teams t
CROSS JOIN LATERAL (
    SELECT 1
    FROM teams t JOIN teams u USING (id) AS o
    CROSS JOIN (
        SELECT 1
        FROM (VALUES (1)) v(n)
        JOIN accounts a ON a.id = o.id
        JOIN accounts b ON b.id = t.id
    ) s
) q;

SELECT 1
FROM orders o
CROSS JOIN LATERAL (
    SELECT 1
    FROM teams t JOIN teams u USING (id) AS o
    CROSS JOIN LATERAL (
        SELECT 1
        FROM (VALUES (1)) v(n)
        JOIN accounts a ON a.id = o.id
        JOIN accounts b ON b.id = t.id
    ) s
) q;

SELECT s.name, s.email
FROM orders o
LEFT JOIN LATERAL (
    SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id) AS name,
           (SELECT b.email FROM accounts b WHERE b.id = o.account_id) AS email
    OFFSET 0
) s ON FALSE;

SELECT s.name, s.email
FROM orders o
LEFT JOIN LATERAL (
    SELECT (SELECT a.name FROM accounts a WHERE a.id = o.account_id) AS name,
           (SELECT b.email FROM accounts b WHERE b.id = o.account_id) AS email
    OFFSET 0
) s ON o.enabled;

SELECT x.*
FROM orders o
CROSS JOIN XMLTABLE(
    '/r' PASSING '<r><v>1</v></r>'::xml
    COLUMNS ordinal FOR ORDINALITY,
            p text PATH (SELECT a.path FROM paths a WHERE a.id = o.path_id),
            q text PATH (SELECT b.path FROM paths b WHERE b.id = o.path_id)
) AS x;

SELECT pg_catalog.row_number() OVER (
    PARTITION BY o.id
    ROWS BETWEEN (SELECT a.n FROM accounts a WHERE a.id = $1) PRECEDING
         AND (SELECT b.n FROM accounts b WHERE b.id = $1) FOLLOWING
)
FROM orders o
GROUP BY o.id;

SELECT pg_catalog.row_number() OVER (
    PARTITION BY o.id
    ROWS BETWEEN (SELECT a.n FROM accounts a WHERE a.id = $1) PRECEDING
         AND (SELECT b.n FROM accounts b WHERE b.id = $1) FOLLOWING
), pg_catalog.sum(o.id) OVER (
    PARTITION BY o.id
    ROWS BETWEEN (SELECT a.n FROM accounts a WHERE a.id = $1) PRECEDING
         AND (SELECT b.n FROM accounts b WHERE b.id = $1) FOLLOWING
)
FROM orders o
GROUP BY o.id;

SELECT nested.id
FROM orders o
LEFT JOIN (
    accounts a JOIN accounts b ON b.id = a.id
    JOIN accounts c ON c.id = a.id
) AS nested ON FALSE;
