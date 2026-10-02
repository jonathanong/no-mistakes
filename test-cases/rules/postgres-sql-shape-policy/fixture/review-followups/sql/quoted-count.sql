SELECT "count"(*) > 0 FROM orders;
SELECT pg_catalog."count"(*) > 0 FROM orders;
SELECT "pg_catalog"."count"(*) > 0 FROM orders;
-- Quoted case and a dot inside one identifier are not the lowercase built-in.
SELECT "COUNT"(*) > 0 FROM orders;
SELECT "PG_CATALOG"."count"(*) > 0 FROM orders;
SELECT "pg_catalog.count"(*) > 0 FROM orders;
SELECT app."count"(*) > 0 FROM orders;
SELECT a.b.count(*) > 0 FROM orders;
