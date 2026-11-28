UPDATE orders o SET enabled = false WHERE o.enabled
RETURNING CASE WHEN old.enabled THEN '' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = old.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = old.account_id)
END;

UPDATE orders o SET enabled = false WHERE o.enabled
RETURNING CASE WHEN o.enabled THEN '' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = o.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = o.account_id)
END;

UPDATE orders o SET enabled = false WHERE o.enabled
RETURNING CASE WHEN new.enabled THEN '' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = new.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = new.account_id)
END;

UPDATE orders o SET enabled = false WHERE o.enabled
RETURNING CASE WHEN old.enabled THEN
    (SELECT a.name FROM accounts a WHERE a.id = old.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = old.account_id)
ELSE '' END;

MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN MATCHED AND s.enabled THEN UPDATE SET account_name = ''
RETURNING CASE WHEN s.enabled THEN '' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = s.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = s.account_id)
END;

MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN MATCHED AND s.enabled THEN UPDATE SET account_name = ''
RETURNING CASE WHEN s.enabled THEN
    (SELECT a.name FROM accounts a WHERE a.id = s.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = s.account_id)
ELSE '' END;

MERGE INTO orders o USING incoming s ON o.id = s.id AND o.enabled
WHEN MATCHED AND s.enabled THEN UPDATE SET enabled = false
RETURNING CASE WHEN o.enabled THEN '' ELSE
    (SELECT a.name FROM accounts a WHERE a.id = s.account_id) ||
    (SELECT b.name FROM accounts b WHERE b.id = s.account_id)
END;

SELECT CASE WHEN NOT o.enabled THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
ELSE 0 END
FROM orders o
INNER JOIN teams t ON o.enabled;

SELECT CASE WHEN o.enabled THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
ELSE 0 END
FROM orders o
INNER JOIN teams t ON o.enabled;

SELECT CASE WHEN NOT o.enabled THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
ELSE 0 END
FROM orders o
LEFT JOIN teams t ON o.enabled;

SELECT CASE WHEN NOT o.enabled THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
ELSE 0 END
FROM (orders o INNER JOIN teams t ON o.enabled)
RIGHT JOIN regions r ON r.id = o.region_id;

SELECT CASE WHEN NOT o.enabled THEN
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
ELSE 0 END
FROM (orders o INNER JOIN teams t ON o.enabled)
FULL OUTER JOIN regions r ON r.id = o.region_id;

SELECT
    (SELECT a.n FROM accounts a WHERE a.id = o.id) +
    (SELECT b.n FROM accounts b WHERE b.id = o.id)
FROM orders o
LEFT JOIN (teams t INNER JOIN regions r ON t.enabled AND NOT t.enabled)
    ON t.id = o.team_id;

SELECT COALESCE(
    o.enabled,
    CASE WHEN o.enabled IS NOT NULL THEN (
        SELECT
            (SELECT a.n FROM accounts a WHERE a.id = o.id) +
            (SELECT b.n FROM accounts b WHERE b.id = o.id)
    ) > 0 ELSE false END
)
FROM orders o;

SELECT COALESCE(
    o.enabled,
    CASE WHEN o.enabled IS NULL THEN (
        SELECT
            (SELECT a.n FROM accounts a WHERE a.id = o.id) +
            (SELECT b.n FROM accounts b WHERE b.id = o.id)
    ) > 0 ELSE false END
)
FROM orders o;

SELECT COALESCE(
    o.enabled,
    pg_catalog.sum((
        SELECT
            (SELECT a.n FROM accounts a WHERE a.id = o.id) +
            (SELECT b.n FROM accounts b WHERE b.id = o.id)
    )) > 0
)
FROM orders o
WHERE o.enabled
GROUP BY o.enabled, o.id;

WITH candidate AS MATERIALIZED (
    SELECT
        (SELECT a.n FROM accounts a WHERE a.id = o.id) AS x,
        (SELECT b.n FROM accounts b WHERE b.id = o.id) AS y
    FROM orders o
)
SELECT COALESCE(0, (SELECT x FROM candidate));

WITH candidate AS MATERIALIZED (
    SELECT
        (SELECT a.n FROM accounts a WHERE a.id = o.id) AS x,
        (SELECT b.n FROM accounts b WHERE b.id = o.id) AS y
    FROM orders o
)
SELECT COALESCE(NULL, (SELECT x FROM candidate));

WITH changed AS (
    DELETE FROM orders o
    RETURNING
        (SELECT a.n FROM accounts a WHERE a.id = o.id) AS x,
        (SELECT b.n FROM accounts b WHERE b.id = o.id) AS y
)
SELECT COALESCE(0, (SELECT x FROM changed));

SELECT EXISTS (
    SELECT
        (SELECT a.n FROM accounts a WHERE a.id = o.id),
        (SELECT b.n FROM accounts b WHERE b.id = o.id)
    FROM orders o
    LIMIT 1.0
);

SELECT EXISTS (
    SELECT
        (SELECT a.n FROM accounts a WHERE a.id = o.id),
        (SELECT b.n FROM accounts b WHERE b.id = o.id)
    FROM orders o
    LIMIT 0.5
);

SELECT EXISTS (
    SELECT
        (SELECT a.n FROM accounts a WHERE a.id = o.id),
        (SELECT b.n FROM accounts b WHERE b.id = o.id)
    FROM orders o
    LIMIT 1e2
);

SELECT EXISTS (
    SELECT
        (SELECT a.n FROM accounts a WHERE a.id = o.id),
        (SELECT b.n FROM accounts b WHERE b.id = o.id)
    FROM orders o
    LIMIT 0.4
);

SELECT EXISTS (
    SELECT
        (SELECT a.n FROM accounts a WHERE a.id = o.id),
        (SELECT b.n FROM accounts b WHERE b.id = o.id)
    FROM orders o
    LIMIT -1.0
);

SELECT EXISTS (
    SELECT
        (SELECT a.n FROM accounts a WHERE a.id = o.id),
        (SELECT b.n FROM accounts b WHERE b.id = o.id)
    FROM orders o
    LIMIT 99999999999999999999.0
);

SELECT EXISTS (
    SELECT
        (SELECT a.n FROM accounts a WHERE a.id = o.id),
        (SELECT b.n FROM accounts b WHERE b.id = o.id)
    FROM orders o
    LIMIT 9223372036854775807.5
);
