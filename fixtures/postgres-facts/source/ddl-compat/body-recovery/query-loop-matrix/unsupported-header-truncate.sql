-- An uncertain header must not hide an independently valid action.
DO $body$DECLARE r integer;
BEGIN
    FOR r IN SELECT 1 FOR SHARE LOOP
        TRUNCATE accounts;
    END LOOP;
END
$body$;
