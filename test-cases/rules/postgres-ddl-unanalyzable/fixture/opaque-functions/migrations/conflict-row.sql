CREATE FUNCTION conflict_row() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
INSERT INTO items (a, b, col) SELECT a, b, col FROM staged
ON CONFLICT (col) WHERE col IS NOT NULL
DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b
WHERE (items.a, items.b) IS DISTINCT FROM (EXCLUDED.a, EXCLUDED.b);
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
