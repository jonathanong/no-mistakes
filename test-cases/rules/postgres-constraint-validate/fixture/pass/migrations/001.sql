ALTER TABLE comments ADD CONSTRAINT comments_body_check CHECK (body <> '') NOT VALID;
ALTER TABLE comments VALIDATE CONSTRAINT comments_body_check;
