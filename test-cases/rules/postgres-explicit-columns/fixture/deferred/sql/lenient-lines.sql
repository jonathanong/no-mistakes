ALTER TABLE ignored ADD COLUMN n int GENERATED ALWAYS AS (1) VIRTUAL;

-- no-mistakes-disable-next-line postgres-explicit-columns
SELECT * FROM orders;
