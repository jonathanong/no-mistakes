-- FROM factors are collected before earlier JOIN ON queries; TABLE identities must still follow their source.
WITH "Ids" AS (SELECT id FROM accounts WHERE id = $1)
DELETE FROM accounts WHERE id IN (
  SELECT later.id FROM accounts a JOIN (SELECT 1 AS marker) b
    ON a.id IN (TABLE "Ids")
    JOIN (TABLE Ids LIMIT 1) later ON true WHERE a.id = $1 LIMIT 1
);
-- Retained TABLE set arms exercise the prepared cursor instead of synthetic SELECT identities.
WITH "Ids" AS (SELECT id FROM accounts WHERE id = $1)
DELETE FROM accounts WHERE id IN (
  SELECT later.id FROM accounts a JOIN (SELECT 1 AS marker) b
    ON a.id IN (SELECT id FROM "Ids" UNION ALL TABLE "Ids")
    JOIN (SELECT '00000000-0000-0000-0000-000000000001'::uuid AS id UNION ALL TABLE Ids LIMIT 1) later ON true WHERE a.id = $1 LIMIT 1
);
-- An earlier uncapped physical read cannot disappear behind a later local CTE reference.
WITH "Ids" AS (SELECT '00000000-0000-0000-0000-000000000001'::uuid AS id)
DELETE FROM accounts WHERE id IN (
  SELECT later.id FROM accounts a JOIN (SELECT 1 AS marker) b
    ON a.id IN (TABLE Ids)
    JOIN (TABLE "Ids" LIMIT 1) later ON true WHERE a.id = $1 LIMIT 1
);
-- The same required physical read survives when it is a retained right TABLE arm.
WITH "Ids" AS (SELECT '00000000-0000-0000-0000-000000000001'::uuid AS id)
DELETE FROM accounts WHERE id IN (
  SELECT later.id FROM accounts a JOIN (SELECT 1 AS marker) b
    ON a.id IN (SELECT '00000000-0000-0000-0000-000000000001'::uuid AS id UNION ALL TABLE Ids)
    JOIN (SELECT id FROM "Ids" UNION ALL TABLE "Ids" LIMIT 1) later ON true WHERE a.id = $1 LIMIT 1
);
-- A standalone physical TABLE read remains an offender; outer page caps above retain their existing semantics.
TABLE Ids;
