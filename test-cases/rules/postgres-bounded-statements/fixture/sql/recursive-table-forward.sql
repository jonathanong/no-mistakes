-- Recursive WITH exposes the later accounts alias inside the earlier TABLE arm.
WITH RECURSIVE first AS (TABLE accounts),
  accounts AS (SELECT * FROM public.accounts)
SELECT * FROM first;
