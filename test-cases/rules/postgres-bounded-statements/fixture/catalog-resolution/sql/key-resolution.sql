-- The unqualified key has one catalog owner, and the join key bounds both relations.
SELECT 1 FROM public.customer_accounts AS c JOIN public.owner_jobs AS j ON j.customer_id = c.id WHERE email_intake_id = $1;
SELECT 1 FROM public.customer_accounts AS c JOIN public.owner_jobs AS j ON j.customer_id = c.id WHERE c.email_intake_id = $1;
-- The same unqualified name on two sources cannot be assigned to either key.
SELECT 1 FROM public.ambiguous_left AS l JOIN public.ambiguous_right AS r ON l.id = r.id WHERE email_intake_id = $1;
-- An unknown source could also expose email_intake_id.
SELECT 1 FROM public.customer_accounts AS c JOIN external_email_source AS e ON true WHERE email_intake_id = $1;
-- An opaque table function output alias is also a possible owner.
SELECT 1 FROM public.customer_accounts AS c JOIN get_email_intake_ids() AS e(email_intake_id) ON true WHERE email_intake_id = $1;
-- A self-join creates two possible owners even though both aliases share one catalog table.
SELECT 1 FROM public.customer_accounts AS l JOIN public.customer_accounts AS r ON l.id = r.id WHERE email_intake_id = $1;
