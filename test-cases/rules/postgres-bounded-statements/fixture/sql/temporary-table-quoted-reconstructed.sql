-- Lenient concatenation recovers a query whose TABLE spelling is absent from outer tokens.
CREATE TEMP TABLE "Accounts"(id uuid);
$$SELECT NULL::uuid UNION ALL TABLE "Accounts" LIMIT 1$$ || '';
