-- The quoted temporary relation has a different identity from permanent accounts.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT NULL::uuid UNION ALL TABLE "Accounts";

SELECT NULL::uuid UNION ALL TABLE Accounts;
SELECT NULL::uuid UNION ALL TABLE public.accounts;
SELECT NULL::uuid UNION ALL TABLE pg_temp."Accounts";

-- A view over the quoted temporary relation is itself temporary.
CREATE VIEW orders AS SELECT NULL::uuid UNION ALL TABLE "Accounts";
SELECT * FROM orders;

-- An unquoted temporary name shadows only the folded name.
CREATE TEMP TABLE accounts(id uuid);
SELECT NULL::uuid UNION ALL TABLE accounts;
SELECT NULL::uuid UNION ALL TABLE "Accounts";

-- DDL's TABLE token must not steal the following query's quoted spelling.
COMMENT ON TABLE Accounts IS 'permanent';
SELECT NULL::uuid UNION ALL TABLE "Accounts";
