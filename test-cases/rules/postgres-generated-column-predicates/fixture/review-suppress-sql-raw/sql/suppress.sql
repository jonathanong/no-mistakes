SELECT id FROM orders
-- postgres-generated-column-predicates
WHERE created_at <> $1;
SELECT id FROM orders
ORDER BY created_at; -- postgres-generated-column-predicates
