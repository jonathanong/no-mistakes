-- Labels and nested headers must not hide the first DML action.
CREATE FUNCTION query_loop_nested(ids bigint[]) RETURNS void LANGUAGE plpgsql AS $$
DECLARE selected_id bigint; inner_id bigint;
BEGIN
    FOR selected_id IN
        SELECT id FROM accounts WHERE id = ANY(ids) FOR NO KEY UPDATE LOOP
        IF selected_id + THEN FOR inner_id IN
            SELECT id FROM accounts WHERE id = ANY(ids) FOR NO KEY UPDATE LOOP
            UPDATE accounts SET id = abs(id) WHERE id = selected_id;
        END LOOP; END IF;
    END LOOP;
END;
$$;
CREATE TABLE following_query_loop (id bigint);
