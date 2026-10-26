-- Static EXECUTE SQL is recursively projected into migration facts.
DO $$ BEGIN EXECUTE 'ALTER TABLE accounts ADD COLUMN'; END $$;
