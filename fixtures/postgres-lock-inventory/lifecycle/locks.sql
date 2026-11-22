SELECT id FROM jobs FOR UPDATE;
SELECT id FROM jobs FOR UPDATE;
SELECT pg_advisory_lock(42), pg_try_advisory_xact_lock(43);
