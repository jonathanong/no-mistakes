-- END IF is a procedural delimiter inside this peeled DO body.
DO $$
BEGIN
  IF true THEN
    CREATE TABLE audit_log (id bigint PRIMARY KEY);
  END IF;
END
$$;
