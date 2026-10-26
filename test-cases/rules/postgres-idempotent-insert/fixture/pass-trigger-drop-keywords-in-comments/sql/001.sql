CREATE TRIGGER "drop trigger" BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();
-- DROP TRIGGER decoy ON items;
SELECT 'DROP TRIGGER another_decoy ON items';
DROP TRIGGER "drop trigger" ON items;
INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;
