CREATE INDEX before_lock_idx ON sample_child(parent_id);
DO $$ BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE conname = 'sample_constraint'
      AND conrelid = 'sample_child'::regclass
  ) THEN
    LOCK TABLE sample_parent IN SHARE ROW EXCLUSIVE MODE;
    LOCK TABLE sample_child IN SHARE ROW EXCLUSIVE MODE;
    IF NOT EXISTS (
      SELECT 1
      FROM pg_constraint
      WHERE conname = 'sample_constraint'
        AND conrelid = 'sample_child'::regclass
    ) THEN
      ALTER TABLE sample_child
        ADD CONSTRAINT sample_constraint
        FOREIGN KEY (parent_id) REFERENCES sample_parent(id) NOT VALID;
    END IF;
  END IF;
END $$;
CREATE INDEX after_lock_idx ON sample_parent(id);
