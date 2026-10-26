CREATE TRIGGER items_before_insert BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();
INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;
-- This later replacement must not hide the BEFORE trigger at the INSERT.
CREATE OR REPLACE TRIGGER items_before_insert AFTER INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();
