-- An uncertain header must not hide an independently valid action.
DO $body$DECLARE r integer;
BEGIN
    FOR r IN SELECT id FROM accounts FOR NO KEY LOOP
        TRUNCATE accounts;
    END LOOP;
END
$body$;
