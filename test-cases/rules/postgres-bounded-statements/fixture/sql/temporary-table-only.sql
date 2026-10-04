-- ONLY belongs to each TABLE query arm; the quoted temporary table must not fold to accounts.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT NULL::uuid UNION ALL TABLE ONLY "Accounts";
SELECT NULL::uuid UNION ALL TABLE ONLY Accounts;
SELECT NULL::uuid UNION ALL TABLE ONLY public."Accounts";
