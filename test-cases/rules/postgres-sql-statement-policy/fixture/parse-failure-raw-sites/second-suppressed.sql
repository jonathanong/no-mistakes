SELECT 1;
-- The first opaque statement must remain visible.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $1;
SELECT 1;
-- no-mistakes-disable-next-line postgres-sql-statement-policy
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $2;
