-- The explicit public catalog resolves inner bare accounts to the outer table.
DELETE FROM public.accounts WHERE id IN
  (SELECT public.accounts.id FROM accounts LIMIT 1);
