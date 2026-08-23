CREATE TABLE children (
  parent_id uuid REFERENCES parents(id)
);
ALTER TABLE children ADD CONSTRAINT children_parent_id_not_null CHECK (parent_id IS NOT NULL) NOT VALID;
ALTER TABLE children ADD CONSTRAINT fk_children_parent FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE children ADD UNIQUE (parent_id);
