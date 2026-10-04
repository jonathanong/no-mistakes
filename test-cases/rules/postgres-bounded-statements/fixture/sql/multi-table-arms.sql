-- The middle TABLE arm must leave the following UNION ALL SELECT intact.
CREATE TEMP TABLE "Accounts" (id uuid);
TABLE "Accounts" UNION ALL TABLE "Accounts" UNION ALL SELECT id FROM accounts;
