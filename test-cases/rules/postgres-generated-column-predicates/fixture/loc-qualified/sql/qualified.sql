SELECT id FROM public.history WHERE created_at > $1;
SELECT id FROM audit.history WHERE created_at > $1;
