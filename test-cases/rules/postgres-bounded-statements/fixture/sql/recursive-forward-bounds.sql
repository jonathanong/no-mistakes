-- PostgreSQL resolves acyclic forward references in WITH RECURSIVE by dependency order.
WITH RECURSIVE first AS (SELECT * FROM accounts),
  accounts AS (SELECT * FROM public.accounts)
SELECT * FROM first;

-- A longer chain must resolve its leaf before projecting either dependent CTE.
WITH RECURSIVE first AS (SELECT * FROM middle),
  middle AS (SELECT * FROM accounts),
  accounts AS (SELECT * FROM public.accounts)
SELECT * FROM first;
