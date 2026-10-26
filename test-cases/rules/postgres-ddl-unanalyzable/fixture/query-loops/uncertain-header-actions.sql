-- Header uncertainty must not hide an independently valid action.
CREATE TABLE accounts (id integer, derived integer GENERATED ALWAYS AS (id + 1) STORED);
DO $body$DECLARE r integer;
BEGIN
    FOR r IN SELECT 1 FOR SHARE LOOP
        TRUNCATE accounts;
    END LOOP;
END
$body$;
DO $body$DECLARE r integer;
BEGIN
    FOR r IN SELECT id FROM accounts FOR NO KEY LOOP
        TRUNCATE accounts;
    END LOOP;
END
$body$;
DO $body$DECLARE r integer;
BEGIN
    FOR r IN SELECT 1 FOR SHARE LOOP
        UPDATE accounts SET derived = 7;
    END LOOP;
END
$body$;
DO $body$DECLARE r integer;
BEGIN
    FOR r IN SELECT id FROM accounts FOR NO KEY LOOP
        UPDATE accounts SET WHERE id = 1;
    END LOOP;
END
$body$;
