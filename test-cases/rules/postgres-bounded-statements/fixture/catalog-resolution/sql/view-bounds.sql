SELECT 1 FROM public.bounded_customer_view;
SELECT 1 FROM public.customer_accounts AS c JOIN public.bounded_customer_view AS v ON v.customer_id = c.id;
SELECT 1 FROM public.unbounded_archive;
SELECT 1 FROM public.unbounded_archive LIMIT 5;
SELECT 1 FROM public."Mixed Name";
