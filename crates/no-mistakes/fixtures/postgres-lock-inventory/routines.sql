
-- Routine definitions are separate from executable statements in this projection.
CREATE FUNCTION app.reserve_job() RETURNS bigint LANGUAGE SQL AS $$
    SELECT id FROM jobs FOR UPDATE;
$$;

-- The parser represents an inline procedure body as a BEGIN/END block after AS.
CREATE PROCEDURE app.reserve_slot() LANGUAGE SQL AS
BEGIN
    SELECT pg_advisory_xact_lock(2);
END;

SELECT pg_advisory_xact_lock(3);
