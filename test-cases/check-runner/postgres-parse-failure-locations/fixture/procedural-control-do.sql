DO $$
DECLARE
    row_record record;
    item text;
    selected_id integer;
BEGIN
    -- SQL-bearing control chunks must not escape the enclosing dollar body.
    FOR row_record IN SELECT abs(id) FROM accounts LOOP
        NULL;
    END LOOP;
    FOREACH item IN ARRAY ARRAY['accounts'] LOOP
        EXECUTE format('ALTER TABLE %I ADD COLUMN flag boolean', item);
    END LOOP;
    SELECT id INTO selected_id FROM accounts LIMIT 1;
    PERFORM "into" FROM accounts;
    PERFORM abs(id) FROM accounts WHERE id = selected_id FOR KEY SHARE;
END;
$$;
CREATE TABLE following_body (id integer);
SELECT FROM;
ALTER TABLE;
