DO $$
DECLARE parts integer[] := ARRAY[1];
        i integer;
BEGIN
  FOR i IN 1..array_length(parts, 1) - LOOP
    RAISE NOTICE '%', i;
  END LOOP;
END;
$$;
