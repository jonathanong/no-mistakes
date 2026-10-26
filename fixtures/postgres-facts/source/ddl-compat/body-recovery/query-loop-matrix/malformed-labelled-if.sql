-- An IF cannot have an opening label; recovery must keep this malformed prefix.
CREATE FUNCTION query_loop_nested(ids bigint[]) RETURNS void LANGUAGE plpgsql AS $$
DECLARE selected_id bigint; inner_id bigint;
BEGIN
    <<condition>> IF true THEN
    <<loop_tasks>> FOR selected_id IN
        SELECT id FROM accounts WHERE id = ANY(ids) FOR NO KEY UPDATE LOOP
            UPDATE accounts SET id = abs(id) WHERE id = selected_id;
    END LOOP loop_tasks;
    END IF;
END;
$$;
CREATE TABLE following_query_loop (id bigint);
-- Labels before IF are invalid, while the guarded action remains recoverable.
