CREATE EXTENSION IF NOT EXISTS vector SCHEMA public;

CREATE USER supabase_auth_admin NOINHERIT CREATEROLE LOGIN NOREPLICATION;

DO $$ BEGIN CREATE SCHEMA storage; END $$;

DO $$
DECLARE parts text[] := ARRAY['first', 'last'];
        i integer;
BEGIN
  FOR i IN 1..array_length(parts, 1)-1 LOOP
    RAISE NOTICE '%', i;
  END LOOP;
END;
$$;

CREATE TABLE example (
  id integer CONSTRAINT example_pkey PRIMARY KEY
);
