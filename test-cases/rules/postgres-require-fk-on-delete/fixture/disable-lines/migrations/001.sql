DO $$ BEGIN
  CREATE TABLE inline_case (
    parent_id uuid REFERENCES parents(id) -- no-mistakes-disable-line postgres-require-fk-on-delete: fixture exception
  );
END $$;

DO $$ BEGIN
  CREATE TABLE control_case (
    parent_id uuid REFERENCES parents(id)
  );
END $$;
