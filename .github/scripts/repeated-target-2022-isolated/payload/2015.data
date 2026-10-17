DO $$ BEGIN
  ALTER TABLE children ADD CONSTRAINT child_parent_fkey FOREIGN KEY (parent_id)
    REFERENCES parents (id) ON DELETE SET NULL (parent_id + 1) NOT VALID;
END $$;
