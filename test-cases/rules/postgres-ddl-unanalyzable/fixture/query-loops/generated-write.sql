CREATE TABLE loop_generated (
    id bigint PRIMARY KEY,
    derived bigint GENERATED ALWAYS AS (id + 1) STORED
);
CREATE FUNCTION write_generated(ids bigint[]) RETURNS void LANGUAGE plpgsql AS $$
DECLARE selected_id bigint;
BEGIN
    -- Recover this loop without hiding the separate policy finding below.
    FOR selected_id IN
        SELECT id FROM loop_generated
        WHERE id = ANY(ids)
        ORDER BY id FOR NO KEY UPDATE
    LOOP
        UPDATE loop_generated SET derived = 1 WHERE id = selected_id;
    END LOOP;
END;
$$;

-- A definite policy violation remains visible beside the recovered function.
UPDATE loop_generated SET derived = 1 WHERE id = 1;
