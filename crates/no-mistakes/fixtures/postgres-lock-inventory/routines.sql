
-- Routine definitions are separate from executable statements in this projection.
CREATE FUNCTION app.reserve_job() RETURNS bigint LANGUAGE SQL AS $$
    SELECT id FROM jobs FOR UPDATE;
$$;

CREATE FUNCTION app.reserve_slot() RETURNS void
LANGUAGE SQL AS $$
    SELECT pg_advisory_xact_lock(2);
$$;

SELECT pg_advisory_xact_lock(3);
