
-- Routine definitions are separate from executable statements in this projection.
CREATE FUNCTION app.reserve_job() RETURNS bigint LANGUAGE SQL AS $$
    SELECT id FROM jobs FOR UPDATE;
$$;

-- Valid PL/pgSQL body, kept opaque by the executable-SQL projection.
CREATE PROCEDURE app.reserve_slot() LANGUAGE plpgsql AS $$
BEGIN
    PERFORM pg_advisory_xact_lock(2);
END;
$$;

SELECT pg_advisory_xact_lock(3);
