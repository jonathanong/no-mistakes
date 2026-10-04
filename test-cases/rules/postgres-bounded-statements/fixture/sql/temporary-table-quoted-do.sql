-- A recovered DO body must retain the quoted name of its temporary TABLE arm.
DO $$
BEGIN
  CREATE TEMP TABLE "Accounts"(id uuid);
  SELECT NULL::uuid UNION ALL TABLE "Accounts" LIMIT 1;
END
$$;
