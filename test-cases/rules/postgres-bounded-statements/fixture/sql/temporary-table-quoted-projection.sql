-- The ignored projection's quoted TABLE must not mask the derived FROM arm's permanent read.
-- UNION is blocking, so the derived LIMIT does not bound either input arm.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT (SELECT NULL UNION ALL TABLE "Accounts" LIMIT 1)
FROM (SELECT NULL UNION TABLE Accounts LIMIT 1) d;
