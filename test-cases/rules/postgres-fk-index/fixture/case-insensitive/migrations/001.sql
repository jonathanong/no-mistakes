CREATE TABLE comments (
  post_id uuid REFERENCES posts
);
CREATE INDEX comments_post_id_idx ON comments (POST_ID);
