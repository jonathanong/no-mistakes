-- no-mistakes-disable-file postgres-sql-shape-policy
SELECT id FROM orders WHERE id >= uuidv7();
