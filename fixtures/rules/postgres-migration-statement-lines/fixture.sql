-- Distractor references child_id and ADD CHECK must not supply finding lines.
CREATE TABLE first_child (
  child_id uuid
    REFERENCES parent(id)
);
CREATE TABLE second_child (
  child_id uuid
    REFERENCES parent(id)
);
ALTER TABLE first_child
  ADD CONSTRAINT shared_check
    CHECK (child_id IS NOT NULL)
    NOT VALID;
ALTER TABLE second_child
  ADD CONSTRAINT shared_check
    CHECK (child_id IS NOT NULL)
    NOT VALID;
ALTER TABLE first_child
  VALIDATE CONSTRAINT shared_check;
ALTER TABLE second_child
  VALIDATE CONSTRAINT shared_check;
ALTER TABLE first_child
  ADD COLUMN repeated_id uuid;
ALTER TABLE second_child
  ADD COLUMN repeated_id uuid;
ALTER TABLE first_child ADD CHECK (child_id IS NOT NULL);
ALTER TABLE second_child ADD CHECK (child_id IS NOT NULL);
-- CREATE INDEX ignored ON first_child(child_id);
SELECT 'CREATE INDEX ignored_again ON first_child(child_id)';
CREATE
  INDEX repeated_index ON first_child(child_id);
CREATE
  INDEX repeated_index ON second_child(child_id);
DROP
  INDEX repeated_index;
DROP
  INDEX repeated_index;
DROP
  TABLE first_child;
DROP
  TABLE second_child;
