-- The DO wrapper forces lenient recovery of the following analyzed EXPLAIN.
CREATE TEMP TABLE "Accounts" (id uuid);
DO $$ BEGIN CREATE TYPE ignored_mood AS ENUM ('a'); END $$;
EXPLAIN ANALYZE TABLE "Accounts";
EXPLAIN ANALYZE TABLE Accounts;
