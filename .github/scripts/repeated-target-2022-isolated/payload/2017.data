DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'child_parent_fkey') THEN
    LOCK TABLE parents IN SHARE ROW EXCLUSIVE MODE;
    LOCK TABLE children IN SHARE ROW EXCLUSIVE MODE;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'child_parent_fkey') THEN
      ALTER TABLE children ADD CONSTRAINT child_parent_fkey
        FOREIGN KEY (parent_id) REFERENCES parents (id)
        ON DELETE SET NULL (parent_id) NOT VALID;
    END IF;
  END IF;
END $$;

DO $$ BEGIN
  ALTER TABLE children ADD CONSTRAINT child_parent_default_fkey
    FOREIGN KEY (parent_id, tenant_id) REFERENCES parents (id, tenant_id)
    ON DELETE SET DEFAULT (parent_id) NOT VALID;
END $$;

DO $$ BEGIN
  ALTER TABLE children ADD COLUMN another_parent_id uuid
    REFERENCES parents (id) ON DELETE SET NULL (another_parent_id);
END $$;
