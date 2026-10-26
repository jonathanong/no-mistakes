-- Unsupported statements must not consume a later parsed statement's ordinal.
CREATE INDEX ignored_index ON;
CREATE
  INDEX repeated_index ON first_child(child_id);
CREATE INDEX ignored_index ON;
CREATE
  INDEX repeated_index ON second_child(child_id);
DROP INDEX;
DROP
  INDEX repeated_index;
DROP TABLE;
DROP
  TABLE first_child;
