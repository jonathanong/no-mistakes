CREATE TABLE comments (
  post_id uuid REFERENCES posts,
  author_id uuid
);
CREATE INDEX comments_post_partial ON comments (post_id) WHERE author_id IS NOT NULL;
