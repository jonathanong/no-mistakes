CREATE TABLE comments (
  post_id uuid REFERENCES posts -- skip-fk
);
