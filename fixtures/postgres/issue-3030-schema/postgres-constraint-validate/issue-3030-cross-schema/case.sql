ALTER TABLE alpha.items ADD CONSTRAINT body_check CHECK (body <> '') NOT VALID;
ALTER TABLE beta.items VALIDATE CONSTRAINT body_check;
