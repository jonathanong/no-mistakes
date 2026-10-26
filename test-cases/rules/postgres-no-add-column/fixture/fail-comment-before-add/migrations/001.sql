-- add attribute
ALTER TABLE posts ADD CONSTRAINT posts_id_check CHECK (id > 0);
ALTER TABLE posts ADD COLUMN status text;
