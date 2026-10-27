CREATE TABLE comments (
  post_id uuid REFERENCES posts
);
CREATE INDEX comments_post_id_hash ON comments USING hash (post_id);
