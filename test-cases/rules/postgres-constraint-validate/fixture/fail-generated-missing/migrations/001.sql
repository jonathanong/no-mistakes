-- The implicit name is predictable even when validation never follows.
ALTER TABLE nv_child_2011 ADD CHECK (d > DATE '2011-01-01') NOT VALID;
