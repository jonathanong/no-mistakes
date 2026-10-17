-- A conflict on a+b does not prove the individual values of a or b match.
-- Replacing only a can move the old row's key and let the next replay insert.
INSERT INTO items (a, b) VALUES (2, 1)
ON CONFLICT ((a + b)) DO UPDATE SET a = EXCLUDED.a;
