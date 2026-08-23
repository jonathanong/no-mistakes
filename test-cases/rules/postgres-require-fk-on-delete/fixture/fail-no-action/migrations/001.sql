CREATE TABLE children (
  parent_id uuid REFERENCES parents(id) ON DELETE NO ACTION
);
