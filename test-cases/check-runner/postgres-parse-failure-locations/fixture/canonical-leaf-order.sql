SELECT 1;
DO $outer$
BEGIN;
  IF TRUE THEN
    INSERT INTO events(id) VALUES (;
  END IF;
  DO $inner$
  BEGIN;
    CREATE RECURSIVE VIEW invalid_view AS SELECT 1;
  END
  $inner$;
END
$outer$;
