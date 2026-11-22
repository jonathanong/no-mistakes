DO $body$
DECLARE
    acquired boolean;
BEGIN
    NULL;
    LOCK ONLY public.jobs IN ACCESS EXCLUSIVE MODE NOWAIT;
    SELECT pg_catalog.PG_TRY_ADVISORY_XACT_LOCK(7) INTO acquired;
END
$body$;
SELECT pg_catalog."PG_ADVISORY_LOCK"(7);
SELECT pg_catalog.PG_TRY_ADVISORY_XACT_LOCK(7) INTO acquired;
