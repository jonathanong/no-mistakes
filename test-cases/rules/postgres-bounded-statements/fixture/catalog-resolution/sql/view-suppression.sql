-- no-mistakes-disable-next-line postgres-bounded-statements -- this view is intentionally unbounded here.
SELECT 1 FROM public.unbounded_archive;
