-- Nested row-locked query loops recover the first MERGE action and continue to later statements.
CREATE FUNCTION query_loop_nested_merge(ids bigint[]) RETURNS void LANGUAGE plpgsql AS $$
DECLARE selected_id bigint; inner_id bigint;
BEGIN
    FOR selected_id IN
        SELECT id FROM accounts WHERE id = ANY(ids) FOR UPDATE LOOP
        FOR inner_id IN
            SELECT id FROM accounts WHERE id = ANY(ids) FOR UPDATE LOOP
            MERGE INTO accounts AS t USING accounts AS s ON t.id = s.id
            WHEN MATCHED THEN UPDATE SET id = abs(s.id);
        END LOOP;
    END LOOP;
END;
$$;
CREATE TABLE following_query_loop (id bigint);
