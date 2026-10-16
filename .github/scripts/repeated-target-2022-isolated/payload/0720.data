-- The implicit CTE excludes unqualified self reads, while real relations remain.
CREATE TEMP RECURSIVE VIEW nums(n) AS SELECT n FROM app.seed UNION ALL SELECT n + 1 FROM nums WHERE n < 3;
CREATE TEMPORARY RECURSIVE VIEW "Nums"("N") AS SELECT 1 UNION ALL SELECT "N" + 1 FROM "Nums" WHERE "N" < 3;
CREATE OR REPLACE TEMP RECURSIVE VIEW replaced(n) AS SELECT 1 UNION ALL SELECT n + 1 FROM replaced WHERE n < 3;
CREATE OR REPLACE TEMPORARY RECURSIVE VIEW replaced_long(n) AS SELECT 1 UNION ALL SELECT n + 1 FROM replaced_long WHERE n < 3;
-- A qualified physical name is not shadowed by the implicit unqualified binding.
CREATE RECURSIVE VIEW "App"."Nums"("N") AS SELECT "N" FROM app."Nums" UNION ALL TABLE "Nums";
CREATE VIEW ordinary(n) AS SELECT n FROM ordinary;
DO $$ BEGIN
  IF true THEN
    CREATE TEMP RECURSIVE VIEW nested(n) AS SELECT n FROM app.seed UNION ALL SELECT n + 1 FROM nested;
  END IF;
END $$;
