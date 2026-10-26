-- A uniquely owned boolean primary key bounds one row; a non-key boolean does not.
SELECT 1 FROM public.singleton_flags s JOIN public.owner_jobs j ON j.id = $1 WHERE only_row;
SELECT 1 FROM public.customer_accounts c JOIN public.owner_jobs j ON j.id = $1 WHERE enabled;
