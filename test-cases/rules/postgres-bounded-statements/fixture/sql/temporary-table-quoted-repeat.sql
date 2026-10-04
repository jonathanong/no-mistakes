-- Each identical TABLE arm consumes exactly one matching source token.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT NULL::uuid UNION ALL TABLE "Accounts" UNION ALL TABLE "Accounts";
CREATE VIEW orders AS SELECT NULL::uuid UNION ALL TABLE "Accounts" UNION ALL TABLE "Accounts";
SELECT * FROM orders;
