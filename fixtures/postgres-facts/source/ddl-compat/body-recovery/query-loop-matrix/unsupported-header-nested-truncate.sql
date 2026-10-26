-- An unsupported outer header must preserve the valid action after a labelled inner loop.
DO $body$DECLARE r integer; s integer;
BEGIN
    FOR r IN SELECT 1 FOR SHARE LOOP
        <<inner_loop>> FOR s IN SELECT 1 LOOP
            TRUNCATE accounts;
        END LOOP;
    END LOOP;
END
$body$;
