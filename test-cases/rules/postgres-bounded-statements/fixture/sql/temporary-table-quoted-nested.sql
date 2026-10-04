-- The scalar subquery's quoted TABLE arm must not donate its identity to the outer unquoted arm.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT (SELECT NULL::uuid UNION ALL TABLE "Accounts" LIMIT 1) UNION ALL TABLE Accounts;
