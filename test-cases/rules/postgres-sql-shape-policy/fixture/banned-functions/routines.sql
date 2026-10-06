DO $$ BEGIN
  PERFORM pg_sleep(1);
  SELECT pg_catalog.pg_sleep_for('1 second');
  IF true THEN PERFORM pg_sleep_until(now()); END IF;
  EXECUTE 'SELECT pg_sleep(1)';
END $$;
CREATE FUNCTION wait_for_work() RETURNS void LANGUAGE plpgsql AS $$ BEGIN
  PERFORM pg_sleep(1);
  RETURN pg_sleep_for('1 second');
END $$;
CREATE PROCEDURE wait_for_jobs() LANGUAGE plpgsql AS $$ BEGIN
  SELECT pg_sleep_until(now());
  EXECUTE 'SELECT pg_sleep(1)';
END $$;
SELECT 'PERFORM pg_sleep(1)';
