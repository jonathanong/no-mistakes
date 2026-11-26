DO $$ BEGIN
  ALTER TABLE inline_case ADD FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE NOT VALID; -- no-mistakes-disable-line postgres-require-named-constraints: fixture exception
END $$;

DO $$ BEGIN
  ALTER TABLE control_case ADD FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE NOT VALID;
END $$;
