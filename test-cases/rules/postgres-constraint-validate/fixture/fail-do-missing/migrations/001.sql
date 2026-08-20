DO $$ BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
    WHERE conname = 'comments_body_check' AND conrelid = 'comments'::regclass
  ) THEN
    ALTER TABLE comments ADD CONSTRAINT comments_body_check CHECK (body <> '') NOT VALID;
  END IF;
END $$;
