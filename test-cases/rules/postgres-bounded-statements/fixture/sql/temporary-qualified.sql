-- PostgreSQL permits the pg_temp qualifier when creating temporary relations.
SELECT id INTO TEMP pg_temp.accounts FROM orders LIMIT 1;
SELECT * FROM accounts;
SELECT * FROM pg_temp.accounts;
DROP TABLE pg_temp.accounts;
SELECT * FROM accounts;
CREATE TEMP TABLE pg_temp.accounts (id uuid);
SELECT * FROM accounts;
DROP TABLE accounts;
SELECT * FROM accounts;
-- A permanent INTO destination does not shadow the permanent catalog relation.
SELECT id INTO regular_output FROM orders LIMIT 1;
SELECT * FROM accounts;
