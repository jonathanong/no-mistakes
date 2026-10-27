CREATE TABLE comments (
  post_id uuid REFERENCES posts
);
CREATE INDEX comments_post_partial ON comments (post_id) WHERE post_id IS NOT NULL;
