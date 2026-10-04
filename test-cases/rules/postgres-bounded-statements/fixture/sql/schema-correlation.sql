-- audit.accounts must not hide an outer public.accounts reference.
DELETE FROM public.accounts WHERE public.accounts.id IN (SELECT public.accounts.id FROM audit.accounts LIMIT 1);
-- A truly local qualified reference is still independent of the target row.
DELETE FROM public.accounts WHERE id IN (SELECT audit.accounts.id FROM audit.accounts LIMIT 1);
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM public.accounts LIMIT 1);
-- With an explicit public catalog, bare inner accounts is the same relation as public.accounts.
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM accounts LIMIT 1);
-- An explicit alias hides the original relation name.
DELETE FROM public.accounts a WHERE a.id IN (SELECT a.id FROM audit.accounts b LIMIT 1);
