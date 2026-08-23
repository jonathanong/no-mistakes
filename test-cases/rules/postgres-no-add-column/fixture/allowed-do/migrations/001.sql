DO $$ BEGIN
  ALTER TABLE posts ADD COLUMN status text NOT NULL DEFAULT 'draft';
END $$;
