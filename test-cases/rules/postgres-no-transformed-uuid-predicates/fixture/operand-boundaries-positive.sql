-- Each statement still transforms its UUID value, including nested predicate and CASE branches.
SELECT 1 FROM accounts WHERE CAST(id::text = $1 AS boolean);
SELECT 1 FROM accounts WHERE COALESCE(id::text = $1, false);
SELECT 1 FROM accounts WHERE CAST(CASE WHEN id = $1 THEN id ELSE $1::uuid END AS text) = $2;
SELECT 1 FROM accounts WHERE lower(CASE id WHEN $1 THEN id::text ELSE email END) = $2;
SELECT 1 FROM accounts WHERE CAST(CASE WHEN uuid_extract_timestamp(id) > $1 THEN email ELSE email END AS text) = $2;
SELECT 1 FROM accounts WHERE COALESCE((uuid_extract_timestamp(id) + INTERVAL '1 day') > $1, false);
SELECT 1 FROM accounts WHERE CAST(id::text || $1 AS text) = $2;
SELECT 1 FROM accounts WHERE CAST(NOT (id::text = $1) AS boolean);
SELECT 1 FROM accounts WHERE COALESCE(id::text IN ($1, $2), false);
SELECT 1 FROM accounts WHERE COALESCE(id::text LIKE $1, false);
SELECT 1 FROM accounts WHERE CAST(id::text IS NOT NULL AS boolean);
SELECT 1 FROM accounts WHERE uuid_extract_timestamp(id) BETWEEN $1 AND $2;
