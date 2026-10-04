-- A table function is independent of a temporary table with the same bare name.
CREATE TEMP TABLE generate_series(id integer);
CREATE VIEW accounts AS SELECT * FROM generate_series(1, 2) AS g(id);
SELECT * FROM accounts;
DROP TABLE generate_series CASCADE;
SELECT * FROM accounts;
