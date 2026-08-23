DO $$ BEGIN
  ALTER TABLE posts ADD COLUMN status text;
END $$;
