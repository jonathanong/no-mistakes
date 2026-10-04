-- Original branch storage metadata must survive typed nested IF projection.
DO $$ BEGIN
IF true THEN
  IF false THEN
    CREATE TABLE conditional_virtual (base integer, derived integer GENERATED ALWAYS AS (base + 1) VIRTUAL);
  ELSE
    CREATE TABLE conditional_stored (base integer, derived integer GENERATED ALWAYS AS (base + 2) STORED);
  END IF;
ELSE
  CREATE TABLE conditional_default (base integer, derived integer GENERATED ALWAYS AS (base + 3));
END IF;
END $$;
