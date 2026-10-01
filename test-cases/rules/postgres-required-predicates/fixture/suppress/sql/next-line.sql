-- no-mistakes-disable-next-line postgres-required-predicates: reporting query
SELECT id FROM events WHERE kind = 'login';
