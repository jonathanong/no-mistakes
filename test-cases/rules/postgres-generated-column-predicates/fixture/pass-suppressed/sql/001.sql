-- no-mistakes-disable-file postgres-generated-column-predicates
SELECT id FROM orders WHERE created_at > $1;
