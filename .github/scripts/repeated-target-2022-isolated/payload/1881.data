-- All branches are source occurrences, never proof that their DDL executes.
/* 雪 */ DO $migration$ BEGIN
IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'children_parent_id_fkey') THEN
  ALTER TABLE children ADD CONSTRAINT children_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES parents(id);
ELSIF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'children_parent_id_fkey') THEN
  ALTER TABLE children VALIDATE CONSTRAINT children_parent_id_fkey;
ELSE
  ALTER TABLE children ADD CONSTRAINT children_id_check CHECK (id IS NOT NULL) NOT VALID;
END IF;
ALTER TABLE children VALIDATE CONSTRAINT children_parent_id_fkey;
END $migration$;
