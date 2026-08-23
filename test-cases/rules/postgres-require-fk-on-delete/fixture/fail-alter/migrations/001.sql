ALTER TABLE children ADD CONSTRAINT fk_children_parent FOREIGN KEY (parent_id) REFERENCES parents(id);
