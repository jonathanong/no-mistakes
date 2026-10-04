-- Rewriting a derived TABLE arm must retain the quoted temporary identity.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT 1 FROM (SELECT NULL UNION TABLE "Accounts") d;
SELECT 1 FROM (SELECT NULL UNION TABLE Accounts) d;
