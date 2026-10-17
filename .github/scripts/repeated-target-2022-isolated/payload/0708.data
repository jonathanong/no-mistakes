DO $$ BEGIN
  ALTER TABLE children ADD CONSTRAINT child_parent_fkey FOREIGN KEY (parent_id)
    REFERENCES parents (id) ON UPDATE SET NULL (parent_id) NOT VALID;
END $$;
