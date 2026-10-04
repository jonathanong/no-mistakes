-- A quoted standalone TABLE remains temporary before a later SELECT FROM.
CREATE TEMP TABLE "Accounts" (id uuid);
TABLE "Accounts" UNION ALL SELECT 1 FROM accounts;
