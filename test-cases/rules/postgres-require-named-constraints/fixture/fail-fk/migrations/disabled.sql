-- no-mistakes-disable-next-line postgres-require-named-constraints: paired later
ALTER TABLE children ADD FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE NOT VALID;
