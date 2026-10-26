-- Physical statement columns, not line numbers, define installation order.
CREATE TRIGGER t BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit(); INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING; DROP TRIGGER t ON items;
