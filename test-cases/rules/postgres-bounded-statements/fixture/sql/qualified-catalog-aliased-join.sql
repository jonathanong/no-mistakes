-- The join alias hides child base names: this qualified read belongs to the outer write.
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM (accounts JOIN orders ON true) AS j LIMIT 1);
-- Without a join alias, the same catalog-qualified read remains local.
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM (accounts JOIN orders ON true) LIMIT 1);
-- A nested derived query has its own namespace, so the enclosing join alias cannot hide its locals.
DELETE FROM public.accounts WHERE id IN (SELECT j.id FROM ((SELECT public.accounts.id FROM accounts) a JOIN (SELECT 1 AS other_id) b ON true) AS j LIMIT 1);
-- Nested join aliases still hide the original child relation name.
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM ((accounts JOIN orders ON true) AS j JOIN sessions ON true) AS k LIMIT 1);
-- A separately visible source of the same relation restores local qualified ownership.
DELETE FROM public.accounts WHERE id IN (SELECT public.accounts.id FROM (accounts JOIN orders ON true) AS j, accounts LIMIT 1);
