-- A valid loop header still reports an unknown SQL query payload.
DO $$
DECLARE
  loop_id integer;
BEGIN
  FOR loop_id IN FROBULATE accounts LOOP
    NULL;
  END LOOP;
END
$$;
