CREATE SCHEMA storage;
DO $$ BEGIN CREATE SCHEMA storage; END$$;
DO $$ BEGIN CREATE SCHEMA IF NOT EXISTS other_storage; END$$;
CREATE FUNCTION storage.prefixes(parts text[]) RETURNS void LANGUAGE plpgsql AS $$
BEGIN
FOR i IN 1..array_length(parts,1)-1 LOOP RAISE NOTICE '%',i; END LOOP;
END$$;
CREATE PROCEDURE storage.prefixes_proc(parts text[]) LANGUAGE plpgsql AS $$
BEGIN
FOR i IN REVERSE 1..array_length(parts,1)-1 BY 2 LOOP RAISE NOTICE '%',i; END LOOP;
END$$;
DO $$ BEGIN FOR i IN 1..2 LOOP RAISE NOTICE '%',i; END LOOP; END$$;
CREATE TABLE following_ranges (id integer);
