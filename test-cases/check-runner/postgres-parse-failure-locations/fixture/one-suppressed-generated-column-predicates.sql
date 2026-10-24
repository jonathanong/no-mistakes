SELECT 1;
-- no-mistakes-disable-next-line postgres-generated-column-predicates
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $1;
SELECT 1;
-- The unsuppressed sibling remains visible at its own start.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $2;
