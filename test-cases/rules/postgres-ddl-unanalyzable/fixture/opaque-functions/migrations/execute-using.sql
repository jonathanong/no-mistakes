CREATE FUNCTION execute_using() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
EXECUTE 'DELETE FROM items WHERE id = $3::uuid AND NOT EXISTS (SELECT 1 FROM (' || edge_query || ' ORDER BY s.id DESC LIMIT 1) remaining)' USING a, b, c;
RETURN NEW;
END
$$;

CREATE TABLE children (parent_id uuid REFERENCES parents(id));
ALTER TABLE children ADD COLUMN extra text;
ALTER TABLE children ADD CHECK (extra IS NOT NULL);
ALTER TABLE children ADD CONSTRAINT children_extra_check CHECK (extra IS NOT NULL) NOT VALID;
CREATE INDEX children_extra_idx ON children (extra);
CREATE INDEX children_extra_parent_idx ON children (extra, parent_id);
CREATE TABLE this_table_name_is_longer_than_the_postgres_identifier_limit_of_sixty_three_bytes (id int);
