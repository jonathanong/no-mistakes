-- no-mistakes-disable-file postgres-require-fk-on-delete: fixture exception
DO $$ BEGIN
  CREATE TABLE children (
    parent_id uuid REFERENCES parents(id)
  );
END $$;
