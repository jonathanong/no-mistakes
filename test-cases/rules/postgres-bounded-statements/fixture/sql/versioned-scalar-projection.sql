-- Bare names added after PostgreSQL 12 can denote user-defined SETOF functions.
SELECT 1 FROM orders JOIN (SELECT regexp_count($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT regexp_instr($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT regexp_substr($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT gen_random_uuid() AS id) d ON orders.id = d.id;
-- Explicit catalog identity preserves scalar proof without inferring a server version.
SELECT 1 FROM orders JOIN (SELECT pg_catalog.regexp_count($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT pg_catalog.regexp_instr($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT pg_catalog.regexp_substr($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT pg_catalog.gen_random_uuid() AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT public.regexp_count($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT lower(regexp_substr($1, $2)) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT regexp_count($1, $2) AS id LIMIT 1) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT lower($1) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT "pg_catalog"."regexp_count"($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT "PG_CATALOG".regexp_count($1, $2) AS id) d ON orders.id = d.id;
