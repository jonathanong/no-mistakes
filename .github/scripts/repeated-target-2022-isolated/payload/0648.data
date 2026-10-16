DO $$ BEGIN
 IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sample_constraint') THEN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'sample_constraint') THEN
   ALTER TABLE sample_child ADD CONSTRAINT sample_constraint FOREIGN KEY (parent_id) REFERENCES sample_parent(id) NOT VALID;
  END IF;
 END IF;
END $$;
