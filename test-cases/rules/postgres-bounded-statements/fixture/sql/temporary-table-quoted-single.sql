-- A quoted TABLE arm must resolve to the temporary relation, not folded accounts.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT NULL::uuid UNION ALL TABLE "Accounts";
