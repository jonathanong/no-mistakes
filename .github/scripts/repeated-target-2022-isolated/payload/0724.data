-- Unsupported neighbors must not drop valid recursive declarations.
UNSUPPORTED;
CREATE RECURSIVE VIEW nums(n) AS SELECT n FROM app.seed UNION ALL SELECT n + 1 FROM nums WHERE n < 3;
CREATE OR REPLACE TEMPORARY RECURSIVE VIEW "Nums"(n) AS SELECT n FROM app."Nums" UNION ALL TABLE "Nums";
CREATE RECURSIVE VIEW missing AS SELECT 1;
DO $$ BEGIN
  CREATE OR REPLACE TEMP RECURSIVE VIEW nested(n) AS SELECT n FROM app.seed UNION ALL SELECT n + 1 FROM nested;
END $$;
DO $$ BEGIN
  CREATE OR REPLACE RECURSIVE VIEW replaced(n) AS SELECT 1 UNION ALL SELECT n + 1 FROM replaced;
END $$;
-- Guard malformed modifier prefixes during DDL recovery.
DO $$ BEGIN CREATE OR; END $$;
DO $$ BEGIN CREATE OR WRONG VIEW wrong(n) AS SELECT 1; END $$;
DO $$ BEGIN CREATE OR REPLACE; END $$;
DO $$ BEGIN CREATE TEMP missing; END $$;
SELECT 2;
