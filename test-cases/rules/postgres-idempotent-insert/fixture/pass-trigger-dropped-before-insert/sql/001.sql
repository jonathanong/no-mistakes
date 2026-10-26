CREATE TRIGGER items_before_insert
BEFORE INSERT ON items
FOR EACH ROW
EXECUTE FUNCTION audit();

DROP TRIGGER items_before_insert ON items;

INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;
