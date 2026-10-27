-- Wrong-kind drops fail even when temporary identity retains a database condition.
CREATE TEMP TABLE "Audit.Database".pg_temp.accounts(id integer);
DROP MATERIALIZED VIEW accounts CASCADE;
SELECT * FROM accounts;
DROP MATERIALIZED VIEW "Audit.Database".pg_temp.accounts CASCADE;
SELECT * FROM accounts;
DROP TABLE "Audit.Database".pg_temp.accounts;
SELECT * FROM accounts;
CREATE TEMP VIEW "Audit.Database".pg_temp.accounts AS SELECT * FROM public.orders;
DROP MATERIALIZED VIEW pg_temp.accounts;
SELECT * FROM accounts;
DROP VIEW "Audit.Database".pg_temp.accounts;
SELECT * FROM accounts;
