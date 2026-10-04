-- The folded namesake inside a DO body is permanent despite the quoted temp table.
DO $$
BEGIN
  CREATE TEMP TABLE "Accounts"(id uuid);
  SELECT NULL::uuid UNION TABLE Accounts;
END
$$;
