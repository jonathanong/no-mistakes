-- no-mistakes-disable-file postgres-no-add-column: fixture exception
DO $$ BEGIN
  ALTER TABLE posts ADD COLUMN status text;
END $$;
