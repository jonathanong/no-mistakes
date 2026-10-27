DO $$ BEGIN CREATE SCHEMA; END$$;
DO $$ BEGIN CREATE SCHEMA storage unexpected; END$$;
CREATE FUNCTION storage.broken(parts text[]) RETURNS void LANGUAGE plpgsql AS $$
BEGIN
FOR i IN 1..array_length(parts, 1) - LOOP RAISE NOTICE '%', i; END LOOP;
END$$;
