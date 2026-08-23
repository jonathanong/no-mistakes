CREATE TABLE posts (
  id uuid PRIMARY KEY,
  status text
);
ALTER TABLE posts ADD CONSTRAINT posts_status_check CHECK (status <> '');
