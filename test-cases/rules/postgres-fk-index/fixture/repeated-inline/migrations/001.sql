-- Repeated FK identifiers must retain each declaration's own allow directive.
CREATE TABLE first_comments (
  post_id uuid
    REFERENCES posts(id)
);
CREATE TABLE second_comments (
  post_id uuid
    REFERENCES posts(id) -- fk-index-allow: legacy dual-write
);
