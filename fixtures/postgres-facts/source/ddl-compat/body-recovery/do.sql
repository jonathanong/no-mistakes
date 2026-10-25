DO $body$
DECLARE
    first_id bigint;
    second_id bigint;
    rec record;
BEGIN
    NULL;
    SELECT abs(id) INTO STRICT first_id FROM accounts WHERE id = 1;
    SELECT abs(id), abs(id) INTO STRICT first_id, second_id FROM accounts WHERE id = 1;
    FOR rec IN EXECUTE format('SELECT abs(id), created_at AT TIME ZONE ''UTC'' FROM %I', 'accounts') LOOP
        PERFORM abs(1);
    END LOOP;
    FOR first_id IN SELECT abs(id) FROM accounts WHERE id > 0 ORDER BY id FOR NO KEY UPDATE LOOP
        PERFORM abs(2);
    END LOOP;
END
$body$;
CREATE TABLE following_body_recovery (id bigint);
