-- no-mistakes-disable-next-line postgres-generated-column-predicates
SELECT id FROM orders WHERE created_at > $1;
