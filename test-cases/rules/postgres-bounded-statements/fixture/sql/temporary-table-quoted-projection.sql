-- The ignored projection's quoted TABLE must not mask the derived FROM arm's permanent read.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT (SELECT NULL UNION ALL TABLE "Accounts" LIMIT 1)
FROM (SELECT NULL UNION ALL TABLE Accounts LIMIT 1) d;
