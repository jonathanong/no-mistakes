-- A function range name is bare even when its function call is schema-qualified.
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM public.accounts() LIMIT 1);
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM public.accounts() AS f LIMIT 1);
-- A bare range-function name is local, and physical full table names remain local.
DELETE FROM public.accounts WHERE id IN (SELECT accounts.id FROM public.accounts() LIMIT 1);
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM public.accounts LIMIT 1);
