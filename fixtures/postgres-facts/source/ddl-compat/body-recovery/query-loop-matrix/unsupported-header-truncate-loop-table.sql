-- The SQL action may use loop as an unquoted table identifier.
DO $body$DECLARE r integer;
BEGIN
    FOR r IN SELECT 1 FOR SHARE LOOP
        TRUNCATE loop;
    END LOOP;
END
$body$;
