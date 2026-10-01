-- no-mistakes-disable-file postgres-explicit-columns
SELECT * FROM orders WHERE id = $1;
