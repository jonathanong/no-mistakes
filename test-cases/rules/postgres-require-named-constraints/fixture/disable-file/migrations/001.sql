-- no-mistakes-disable-file postgres-require-named-constraints: fixture exception
DO $$ BEGIN
  ALTER TABLE children ADD FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE NOT VALID;
END $$;
