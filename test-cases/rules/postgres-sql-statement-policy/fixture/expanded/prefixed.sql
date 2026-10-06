CREATE FUNCTION first_cluster_change() RETURNS void LANGUAGE plpgsql AS $$ BEGIN
  ALTER DATABASE app SET statement_timeout = '1s';
  IF true THEN ALTER SYSTEM SET statement_timeout = '2s'; END IF;
END $$;
CREATE PROCEDURE first_setting_change() LANGUAGE plpgsql AS $$ BEGIN
  SELECT CASE WHEN true THEN pg_catalog.set_config('session_replication_role', 'replica', true) ELSE 'origin' END;
END $$;
DO $$ BEGIN
  PERFORM CASE WHEN true THEN set_config('session_replication_role', 'origin', true) ELSE 'replica' END;
END $$;
DO $$ BEGIN
  IF pg_catalog.set_config('session_replication_role', 'replica', true) = 'replica' THEN NULL; END IF;
END $$;
