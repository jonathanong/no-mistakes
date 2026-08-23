DO $$ BEGIN
  EXECUTE format('ALTER TABLE %I ADD COLUMN %I text NOT NULL DEFAULT %L', 'posts', 'status', 'draft');
END $$;
