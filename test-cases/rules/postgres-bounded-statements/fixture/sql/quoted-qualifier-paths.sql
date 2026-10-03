-- One quoted identifier containing a dot cannot hide two schema-qualified identifiers.
DELETE FROM public.accounts WHERE public.accounts.id IN (SELECT public.accounts.id FROM "public.accounts" LIMIT 1);
-- The quoted inner relation's own column is genuinely local and remains a bounded pin.
DELETE FROM public.accounts WHERE id IN (SELECT "public.accounts".id FROM "public.accounts" LIMIT 1);
-- The same distinction applies to an outer alias containing a dot.
DELETE FROM accounts AS "public.accounts" WHERE "public.accounts".id IN (SELECT "public.accounts".id FROM public.accounts LIMIT 1);
-- A dotted derived-table alias does not hide the outer schema-qualified relation either.
DELETE FROM public.accounts WHERE public.accounts.id IN (SELECT public.accounts.id FROM (SELECT 1) AS "public.accounts" LIMIT 1);
