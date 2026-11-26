DO $$ BEGIN
  ALTER TABLE inline_case ADD COLUMN status text; -- no-mistakes-disable-line postgres-no-add-column: fixture exception
END $$;

DO $$ BEGIN
  ALTER TABLE control_case ADD COLUMN status text;
END $$;
