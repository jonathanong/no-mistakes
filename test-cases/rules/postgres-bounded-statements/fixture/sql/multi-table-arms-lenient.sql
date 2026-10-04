-- Unsupported DO DDL forces recovery of the later three-arm query.
CREATE TEMP TABLE "Accounts" (id uuid);
DO $$ BEGIN CREATE TYPE ignored_mood AS ENUM ('a'); END $$;
TABLE "Accounts" UNION ALL TABLE "Accounts" UNION ALL SELECT id FROM accounts;
