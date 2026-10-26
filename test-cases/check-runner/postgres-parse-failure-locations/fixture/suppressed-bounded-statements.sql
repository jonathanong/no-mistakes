SELECT 1;
-- no-mistakes-disable-next-line postgres-bounded-statements
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $1;
SELECT 1;
-- no-mistakes-disable-next-line postgres-bounded-statements
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $2;
