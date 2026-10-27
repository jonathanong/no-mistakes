ALTER TABLE ignored ADD COLUMN n int GENERATED ALWAYS AS (1) VIRTUAL;

-- postgres-explicit-columns
SELECT * FROM orders;
