-- An opaque inner relation cannot prove that a qualified outer reference is local.
DELETE FROM public.accounts WHERE id IN
  (SELECT public.accounts.id FROM app.current_accounts() LIMIT 1);
