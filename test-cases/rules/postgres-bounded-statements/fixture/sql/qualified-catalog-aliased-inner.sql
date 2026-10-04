-- An explicit inner alias hides the base name, so this qualified read is outer.
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM accounts a LIMIT 1);
