CREATE SCHEMA family;
CREATE SCHEMA shared;
CREATE TABLE family.parent (id integer);
CREATE TABLE shared.other_parent (label text);
CREATE TABLE family.child () INHERITS (family.parent, shared.other_parent);
CREATE TABLE family.partitioned (id integer PRIMARY KEY) PARTITION BY RANGE (id);
CREATE TABLE family.partitioned_child PARTITION OF family.partitioned
  FOR VALUES FROM (0) TO (100) PARTITION BY RANGE (id);
