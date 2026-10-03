-- The temporary schema determines identity even when CREATE omits the TEMP keyword.
CREATE TABLE pg_temp.accounts (temp_only uuid);
SELECT * FROM accounts;
DROP TABLE accounts;
SELECT * FROM accounts;
CREATE TABLE "pg_temp".accounts (temp_only uuid);
SELECT * FROM accounts;
DROP TABLE "pg_temp".accounts;
SELECT * FROM accounts;
-- Ordinary or differently quoted schemas and literal dots do not create temporary names.
CREATE TABLE public.accounts (id uuid);
SELECT * FROM accounts;
CREATE TABLE "PG_TEMP".accounts (id uuid);
SELECT * FROM accounts;
CREATE TABLE "pg_temp.accounts" (id uuid);
SELECT * FROM accounts;
