-- ELSIF is a procedural marker only at branch boundaries, never a SQL name.
DO $$ BEGIN
IF true THEN
  ALTER TABLE children ADD COLUMN elsif integer;
  ALTER TABLE children ADD COLUMN expression_value integer GENERATED ALWAYS AS (CASE WHEN id = 1 THEN elsif ELSE 0 END) STORED;
ELSIF false THEN
  ALTER TABLE children ADD COLUMN "ELSIF" integer;
END IF;
END $$;
