-- Qualified middle TABLE already parses and must keep its original AST handling.
CREATE TEMP TABLE "Accounts" (id uuid);
TABLE "Accounts" UNION ALL TABLE pg_temp."Accounts" UNION ALL SELECT id FROM accounts;
