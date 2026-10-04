-- The same spelling without quotes names the billing schema's accounts table.
WITH RECURSIVE first AS (
  TABLE billing.accounts UNION ALL SELECT * FROM accounts
), accounts AS (SELECT * FROM public.accounts)
SELECT * FROM first;
