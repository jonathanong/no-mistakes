DO $$ BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
    WHERE conname = 'inline_case_body_check' AND conrelid = 'inline_case'::regclass
  ) THEN
    ALTER TABLE inline_case ADD CONSTRAINT inline_case_body_check CHECK (body <> '') NOT VALID; -- no-mistakes-disable-line postgres-constraint-validate: fixture exception
  END IF;
END $$;

DO $$ BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
    WHERE conname = 'following_case_body_check' AND conrelid = 'following_case'::regclass
  ) THEN
    -- no-mistakes-disable-next-line postgres-constraint-validate: fixture exception
    ALTER TABLE following_case ADD CONSTRAINT following_case_body_check CHECK (body <> '') NOT VALID;
  END IF;
END $$;

DO $$ BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
    WHERE conname = 'control_case_body_check' AND conrelid = 'control_case'::regclass
  ) THEN
    ALTER TABLE control_case ADD CONSTRAINT control_case_body_check CHECK (body <> '') NOT VALID;
  END IF;
END $$;
