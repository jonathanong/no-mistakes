DO $$ BEGIN
  ALTER DATABASE app SET statement_timeout = '1s';
  SET LOCAL session_replication_role = replica;
  PERFORM set_config('session_replication_role', 'origin', true);
  EXECUTE 'CREATE DATABASE scratch TEMPLATE template0';
END $$;
CREATE OR REPLACE FUNCTION change_schema() RETURNS void LANGUAGE plpgsql AS $$ BEGIN
  CREATE SCHEMA inner_schema;
  SET session_replication_role = replica;
  EXECUTE 'DROP SCHEMA inner_schema';
END $$;
CREATE OR REPLACE PROCEDURE change_cluster() LANGUAGE plpgsql AS $$ BEGIN
  ALTER SYSTEM SET statement_timeout = '2s';
  EXECUTE 'DROP DATABASE scratch';
END $$;
-- Only recoverable EXECUTE text is SQL; ordinary strings and comments stay inert.
SELECT 'CREATE DATABASE inert; SET session_replication_role = replica';
DO $$ BEGIN RAISE NOTICE 'DROP DATABASE inert'; END $$;
