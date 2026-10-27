CREATE TABLE comments (
  post_id uuid REFERENCES posts,
  author_id uuid
);
CREATE INDEX comments_author_idx ON comments (author_id);
