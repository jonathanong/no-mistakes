DO $$ BEGIN
  PERFORM pg_sleep(1);
  IF pg_sleep(1) IS NULL THEN
    PERFORM pg_sleep(1);
  END IF;
END $$;
SELECT id FROM items LIMIT 1;
