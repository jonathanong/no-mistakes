-- A top-level function loop must receive the same recovery as nested DO loops.
CREATE FUNCTION loop_rows() RETURNS void LANGUAGE plpgsql AS $$
DECLARE r record;
BEGIN
  FOR r IN EXECUTE 'SELECT 1' LOOP
    NULL;
  END LOOP;
END
$$;
