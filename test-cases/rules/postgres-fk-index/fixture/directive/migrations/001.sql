CREATE TABLE comments (
  id uuid PRIMARY KEY,
  post_id uuid REFERENCES posts -- fk-index-allow: legacy dual-write
);
