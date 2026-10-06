DO $$ BEGIN
  EXECUTE 'SET LOCAL session_replication_role = replica';
  EXECUTE 'SELECT set_config(''session_replication_role'', ''origin'', true)';
END $$;
CREATE FUNCTION change_setting() RETURNS void LANGUAGE plpgsql AS $$ BEGIN
  IF true THEN SET LOCAL session_replication_role = replica; END IF;
END $$;
