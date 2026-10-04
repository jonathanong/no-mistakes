-- An ignored quoted projection must not rename a direct derived TABLE source.
SELECT (SELECT NULL UNION ALL TABLE public."Accounts" LIMIT 1)
FROM (TABLE public.Accounts) d;
