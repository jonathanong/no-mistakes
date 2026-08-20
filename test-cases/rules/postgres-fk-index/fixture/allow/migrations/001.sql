CREATE TABLE comments (
  id uuid PRIMARY KEY,
  post_id uuid REFERENCES posts
);
