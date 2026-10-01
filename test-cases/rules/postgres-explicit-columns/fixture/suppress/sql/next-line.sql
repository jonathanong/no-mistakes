-- no-mistakes-disable-next-line postgres-explicit-columns
SELECT * FROM orders WHERE id = $1;
