-- Recovered and dynamic wrappers must retain their physical statement heads.
DO $$ BEGIN
  CREATE
    TABLE wrapped_first (owner_id uuid REFERENCES owners(id));
  CREATE
    TABLE wrapped_second (owner_id uuid REFERENCES owners(id));
END $$;
DO $$ BEGIN
  EXECUTE 'ALTER TABLE wrapped_first ADD COLUMN repeated_id uuid';
  EXECUTE 'ALTER TABLE wrapped_second ADD COLUMN repeated_id uuid';
END $$;
