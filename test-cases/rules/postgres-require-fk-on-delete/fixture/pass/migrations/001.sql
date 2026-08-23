CREATE TABLE children (
  parent_id uuid REFERENCES parents(id) ON DELETE CASCADE
);
CREATE TABLE leftovers (
  parent_id uuid REFERENCES parents(id) ON DELETE SET NULL
);
CREATE TABLE blocked (
  parent_id uuid REFERENCES parents(id) ON DELETE RESTRICT
);
CREATE TABLE reset (
  parent_id uuid REFERENCES parents(id) ON DELETE SET DEFAULT
);
ALTER TABLE children ADD CONSTRAINT fk_named FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE;
