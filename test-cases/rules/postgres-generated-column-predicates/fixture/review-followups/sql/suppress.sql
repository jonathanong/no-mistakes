SELECT id FROM orders
-- no-mistakes-disable-next-line postgres-generated-column-predicates
WHERE created_at <> $1;
SELECT id FROM orders
ORDER BY created_at; -- no-mistakes-disable-line postgres-generated-column-predicates
