-- CREATE TABLE AS is not a bounded read, so its quoted arm must not supply
-- the spelling of the later unquoted TABLE arm.
CREATE TEMP TABLE "Accounts"(id uuid);
CREATE TABLE snapshot AS TABLE "Accounts";
SELECT NULL::uuid UNION ALL TABLE Accounts;
