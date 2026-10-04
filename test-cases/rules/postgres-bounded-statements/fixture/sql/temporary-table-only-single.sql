-- A strict parse must retain ONLY and the quoted temporary name in one TABLE arm.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT NULL::uuid UNION ALL TABLE ONLY "Accounts";
