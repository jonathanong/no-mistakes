-- Qualified or quoted lookalikes are ordinary functions and may return NULL.
SELECT 1 FROM orders LIMIT app."coalesce"(1);
SELECT 1 FROM orders LIMIT pg_catalog.least(1, 10);
SELECT 1 FROM orders LIMIT "greatest"(1, 10);
-- Actual unqualified built-in constructs retain their fixed caps.
SELECT 1 FROM orders LIMIT COALESCE($1, 10);
SELECT 1 FROM orders LIMIT LEAST($1, 10);
SELECT 1 FROM orders LIMIT GREATEST($1, 10);
