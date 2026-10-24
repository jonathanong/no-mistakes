-- Casts and functions on catalog-resolved UUID key columns in predicates lose
-- the direct UUID comparison that can use the key index.
SELECT id FROM users WHERE id::text = $1;
SELECT id FROM users WHERE CAST(id AS text) = $1;
SELECT id FROM users WHERE uuid_extract_timestamp(id) > $1;
SELECT id FROM users WHERE external_ref::text = $1;

SELECT s.id
FROM sessions s
JOIN users u ON s.user_id::text = u.id::text;

SELECT user_id, count(*)
FROM sessions
GROUP BY user_id
HAVING uuid_extract_timestamp(user_id) > $1;

-- The indexed UUID stays bare when casting the parameter or computing a bound.
SELECT id FROM users WHERE id = $1::uuid;
SELECT id FROM users WHERE id = CAST($1 AS uuid);
SELECT id FROM users WHERE id >= uuidv7_lower_bound($1);

-- A transformed key in the projection does not constrain or sort the scan.
SELECT uuid_extract_timestamp(id) FROM users WHERE id = $1;

SELECT trace_id FROM users WHERE trace_id::text = $1;

-- Plain indexes do not change the UUID typing scope; functions can hide them too.
SELECT trace_id FROM users WHERE uuid_extract_timestamp(trace_id) > $1;

-- Non-UUID keys do not have the UUID comparison contract.
SELECT email FROM users WHERE email::text = $1;
