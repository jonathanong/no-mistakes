-- The DO wrapper forces lenient recovery of following TABLE queries.
CREATE TEMP TABLE "Accounts"(id uuid);
DO $$ BEGIN CREATE TYPE ignored_mood AS ENUM ('a'); END $$;
TABLE "Accounts";
TABLE Accounts;
