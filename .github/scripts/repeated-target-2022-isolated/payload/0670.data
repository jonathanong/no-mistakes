-- Recover safe occurrences while preserving explicit unsupported typed details.
DO $$ BEGIN
 IF true THEN
  ALTER TABLE sample_child DROP COLUMN old_column;
  ALTER TABLE sample_child ADD CONSTRAINT excluded EXCLUDE USING gist (parent_id WITH =);
  SELECT d.id FROM generate_series(1, 5) d;
  INSERT INTO sample_child SELECT d.id FROM generate_series(1, 5) d;
  ALTER TABLE sample_child ADD CONSTRAINT supported CHECK (parent_id > 0);
 END IF;
END $$;
