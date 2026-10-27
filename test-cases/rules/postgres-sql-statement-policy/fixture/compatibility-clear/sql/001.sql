CREATE EXTENSION IF NOT EXISTS vector SCHEMA public;
CREATE USER supabase_auth_admin NOINHERIT CREATEROLE LOGIN NOREPLICATION;
DO $$
BEGIN
  CREATE SCHEMA storage;
END$$;
CREATE FUNCTION storage.prefixes(parts text[]) RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  FOR i IN 1..array_length(parts, 1) - 1 LOOP
    RAISE NOTICE '%', i;
  END LOOP;
END$$;
