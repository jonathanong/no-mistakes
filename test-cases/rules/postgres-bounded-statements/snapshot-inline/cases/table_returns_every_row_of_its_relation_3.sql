SELECT id FROM accounts WHERE id = $1 UNION TABLE public.accounts;
