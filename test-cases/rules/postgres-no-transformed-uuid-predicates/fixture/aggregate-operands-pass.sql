-- Aggregates consume rows, and parameter-side calls leave the UUID column operand bare.
SELECT email FROM accounts GROUP BY email HAVING COUNT(DISTINCT id) > 1;
SELECT email FROM accounts GROUP BY email HAVING MIN(id) = $1;
SELECT email FROM accounts GROUP BY email HAVING MAX(id) >= $1;
SELECT email FROM accounts GROUP BY email HAVING ARRAY_AGG(id) = $1::uuid[];
SELECT email FROM accounts GROUP BY email HAVING JSONB_AGG(id) = $1;
SELECT email FROM accounts GROUP BY email HAVING pg_catalog.count(DISTINCT id) > 1;
SELECT email FROM accounts GROUP BY email HAVING CAST(MIN(id) AS text) = $1;
SELECT email FROM accounts GROUP BY email HAVING lower(MAX(id)::text) = $1;
SELECT email FROM accounts GROUP BY email HAVING COUNT(id) FILTER (WHERE id >= fn($1)) > 1;
SELECT 1 FROM accounts WHERE id >= uuid_from_timestamp($1);
SELECT 1 FROM accounts WHERE uuid_from_timestamp($1) <= id;
SELECT 1 FROM accounts WHERE id BETWEEN uuid_from_timestamp($1) AND uuid_from_timestamp($2);
SELECT 1 FROM accounts a JOIN orders o ON a.id >= uuid_from_timestamp($1);
SELECT 1 FROM accounts WHERE COALESCE(id >= fn($1), false);
SELECT 1 FROM accounts WHERE CAST(id >= fn($1) AS boolean);
SELECT email FROM accounts GROUP BY email HAVING pg_catalog.min(id) = $1;
SELECT email FROM accounts GROUP BY email HAVING "min"(id) = $1;
SELECT email FROM accounts GROUP BY email HAVING COALESCE(MIN(id), $1::uuid) = $2;
-- Even transformed aggregate inputs feed the aggregate result rather than a UUID lookup.
SELECT email FROM accounts GROUP BY email HAVING COUNT(DISTINCT id::text) > 1;
SELECT email FROM accounts GROUP BY email HAVING COUNT(fn(id)) > 1;
SELECT email FROM accounts GROUP BY email HAVING MIN(fn(id)) = $1;
SELECT email FROM accounts GROUP BY email HAVING ARRAY_AGG(id::text) = $1;
SELECT email FROM accounts GROUP BY email HAVING COALESCE(COUNT(fn(id)), 0) > 1;
SELECT email FROM accounts GROUP BY email HAVING pg_catalog.max(fn(id)) = $1;
SELECT email FROM accounts GROUP BY email HAVING COALESCE(COUNT(id) FILTER (WHERE id = $1), 0) > 1;
