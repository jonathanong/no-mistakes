DO $$ BEGIN
  CREATE TABLE children (
    parent_id uuid REFERENCES parents(id)
  );
END $$;
