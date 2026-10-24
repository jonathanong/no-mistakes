SELECT 1 FROM public.customer_accounts AS c JOIN public.owner_jobs AS j ON j.customer_id = c.id WHERE email_intake_id IS NOT DISTINCT FROM $1;
SELECT 1 FROM public.customer_accounts AS c JOIN public.owner_jobs AS j ON j.customer_id = c.id WHERE email_intake_id IN ($1, $2);
SELECT 1 FROM public.customer_accounts AS c JOIN public.owner_jobs AS j ON j.customer_id = c.id WHERE email_intake_id = ANY($1::uuid[]);
SELECT 1 FROM public.customer_accounts AS c JOIN public.owner_jobs AS j ON j.customer_id = c.id WHERE email_intake_id IN (SELECT $1::uuid);
-- A self-comparison does not provide an independent caller-sized value.
SELECT 1 FROM public.customer_accounts AS c JOIN public.owner_jobs AS j ON j.customer_id = c.id WHERE email_intake_id = email_intake_id;
-- A different bare column still depends on a row from the query.
SELECT 1 FROM public.customer_accounts AS c JOIN public.owner_jobs AS j ON j.customer_id = c.id WHERE email_intake_id = customer_id;
