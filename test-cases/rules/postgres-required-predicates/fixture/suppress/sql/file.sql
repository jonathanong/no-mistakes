-- no-mistakes-disable-file postgres-required-predicates
SELECT id FROM events WHERE kind = 'login';
