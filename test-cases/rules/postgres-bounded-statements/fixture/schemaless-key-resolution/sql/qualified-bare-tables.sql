SELECT 1 FROM public.big;
SELECT 1 FROM other.big;
SELECT 1 FROM public.a AS x JOIN public.b AS y ON true WHERE email = $1;
-- A qualified bare-key table still honors its known unique email key.
SELECT 1 FROM public.a WHERE email = $1;
