-- Qualified arms parse as separate set operands and consume their own source tokens.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT NULL::uuid UNION ALL TABLE pg_temp."Accounts" UNION ALL TABLE pg_temp."Accounts";
CREATE VIEW orders AS SELECT NULL::uuid UNION ALL TABLE pg_temp."Accounts" UNION ALL TABLE pg_temp."Accounts";
SELECT * FROM orders;
SELECT NULL::uuid UNION ALL TABLE Accounts;
