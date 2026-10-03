-- Catalog-qualified built-in numeric casts prove zero; application and quoted type names do not.
SELECT * FROM accounts ORDER BY id LIMIT 0::pg_catalog.int8;
SELECT * FROM accounts ORDER BY id LIMIT CAST(0 AS pg_catalog.numeric);
SELECT * FROM accounts ORDER BY id LIMIT 0::PG_CATALOG.FLOAT8;
SELECT * FROM accounts ORDER BY id LIMIT 0::app.int8;
SELECT * FROM accounts ORDER BY id LIMIT 0::"pg_catalog".int8;
SELECT * FROM accounts ORDER BY id LIMIT 0::pg_catalog."int8";
-- A conversion through text is not a transparent numeric-only zero proof.
SELECT * FROM accounts ORDER BY id LIMIT (0::text)::int8;
