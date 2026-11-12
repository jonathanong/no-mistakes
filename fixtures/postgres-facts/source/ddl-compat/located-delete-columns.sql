ALTER TABLE child
  ADD CONSTRAINT first_parent FOREIGN KEY (tenant, parent_id)
  REFERENCES parent(tenant, id) ON DELETE SET NULL (parent_id),
  ADD CONSTRAINT second_parent FOREIGN KEY (owner_id)
  REFERENCES owners(id) ON DELETE SET DEFAULT (owner_id);
