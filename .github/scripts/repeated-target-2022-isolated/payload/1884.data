CREATE TABLE unproven_constraints (
  id integer PRIMARY KEY,
  CONSTRAINT valid_check CHECK (id > 0)
);
ALTER TABLE unproven_constraints ADD CONSTRAINT later_check CHECK (id > 1);
ALTER TABLE unproven_constraints ADD COLUMN later_column integer;
