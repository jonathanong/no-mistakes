-- The scalar subquery's unquoted TABLE arm must not steal the outer quoted temporary arm.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT (SELECT NULL::uuid UNION ALL TABLE Accounts LIMIT 1) UNION ALL TABLE "Accounts";
