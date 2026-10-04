-- A quoted one-part name with a dot is distinct from billing.accounts.
-- Both CTE bodies use TABLE: dependency order visits the later source first,
-- while exact quoted identity must still come from the earlier source token.
WITH RECURSIVE first AS (
  TABLE "billing.accounts" UNION ALL SELECT * FROM accounts
), accounts AS (TABLE public.accounts)
SELECT * FROM first;
