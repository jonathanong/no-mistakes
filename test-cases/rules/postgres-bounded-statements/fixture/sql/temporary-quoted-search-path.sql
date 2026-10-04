CREATE SCHEMA "empty,schema";
CREATE SCHEMA "odd""schema";
CREATE SCHEMA "MiXeD";
CREATE TABLE public.accounts(id uuid PRIMARY KEY);
CREATE TEMP TABLE accounts(id uuid);

-- SQL string values each name one schema; commas and quotes inside are literal.
SET search_path = 'empty,schema', pg_temp, public;
SELECT * FROM accounts;
SET search_path = 'odd"schema', pg_temp, public;
SELECT * FROM accounts;
SET search_path = 'MiXeD', pg_temp, public;
SELECT * FROM accounts;
-- A directly quoted identifier uses SQL identifier escaping.
SET search_path = "empty,schema", pg_temp, public;
SELECT * FROM accounts;
-- A whole list inside one SQL string is one schema; pg_temp is implicit first.
SET search_path = 'empty_schema, pg_temp, public';
SELECT * FROM accounts;
-- Missing evidence and dynamic role substitution remain conservative.
SET search_path = unknown_schema, pg_temp, public;
SELECT * FROM accounts;
SET search_path = '$user', pg_temp, public;
SELECT * FROM accounts;
