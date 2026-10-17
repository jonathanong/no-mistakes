-- Characterized consumer DO bodies are not silently treated as empty DDL.
DO $$ BEGIN
 ALTER TABLE children ADD CONSTRAINT owner FOREIGN KEY (parent_id) REFERENCES parents(id);
 ALTER TABLE children VALIDATE CONSTRAINT owner;
END $$;
CREATE TABLE after_block (id int);
