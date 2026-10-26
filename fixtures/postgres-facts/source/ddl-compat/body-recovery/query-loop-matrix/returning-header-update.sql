-- Each DML RETURNING header and its first write action retain their own calls.
CREATE FUNCTION returning_loop() RETURNS void LANGUAGE plpgsql AS $$
DECLARE selected_id bigint;
BEGIN
    FOR selected_id IN INSERT INTO accounts (id) VALUES (1) RETURNING abs(id) LOOP
        UPDATE accounts SET id = abs(id) WHERE id = selected_id;
    END LOOP;
    FOR selected_id IN UPDATE accounts SET id = 2 RETURNING abs(id) LOOP
        UPDATE accounts SET id = abs(id) WHERE id = selected_id;
    END LOOP;
    FOR selected_id IN DELETE FROM accounts WHERE id = 2 RETURNING abs(id) LOOP
        UPDATE accounts SET id = abs(id) WHERE id = selected_id;
    END LOOP;
END;
$$;
CREATE TABLE following_query_loop (id bigint);
