CREATE TABLE comments (
  id uuid PRIMARY KEY,
  post_id uuid REFERENCES posts
);
CREATE INDEX comments_post_id_idx ON comments (post_id);
