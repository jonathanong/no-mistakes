CREATE TRIGGER n BEFORE UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION normalize();
CREATE TRIGGER t AFTER UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION audit();
DROP TRIGGER n ON items;
-- The dropped normalizer must not invalidate the surviving audit trigger's WHERE proof.
INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')
ON CONFLICT (id) DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b
WHERE items.b IS DISTINCT FROM EXCLUDED.b;
