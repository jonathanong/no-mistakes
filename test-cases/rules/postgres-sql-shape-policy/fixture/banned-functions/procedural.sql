DO $$ BEGIN
  IF CASE WHEN true THEN pg_sleep(1) ELSE NULL END IS NULL THEN NULL; END IF;
END $$;
CREATE FUNCTION wait_condition() RETURNS void LANGUAGE plpgsql AS $$ BEGIN
  IF pg_sleep(1) IS NULL THEN
    SELECT pg_sleep_for('1 second');
  ELSE PERFORM pg_sleep_until(now()); END IF;
  RETURN QUERY SELECT pg_sleep(1);
END $$;
CREATE PROCEDURE wait_cte() LANGUAGE plpgsql AS $$ BEGIN
  WITH timer AS (SELECT pg_sleep(1)) SELECT * FROM timer;
END $$;
