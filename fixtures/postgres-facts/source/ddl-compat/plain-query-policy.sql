DO $$
DECLARE
    selected_id integer;
BEGIN
    NULL;
    -- These plain queries begin a fresh chunk after an earlier statement.
    SELECT abs(id) INTO selected_id FROM accounts FOR KEY SHARE;
    WITH locked AS (SELECT id FROM accounts) SELECT abs(id) INTO selected_id FROM locked FOR KEY SHARE;
END;
$$;
CREATE TABLE following_locked_query (id integer);
