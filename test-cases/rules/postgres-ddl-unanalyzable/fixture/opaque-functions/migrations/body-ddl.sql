CREATE FUNCTION stored_ddl() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
ALTER TABLE hidden ADD COLUMN ignored text;
EXECUTE 'ALTER TABLE hidden ADD COLUMN dynamic_ignored text';
TRUNCATE hidden;
END
$$;
DO $$ BEGIN ALTER TABLE visible ADD COLUMN reported text; END $$;
