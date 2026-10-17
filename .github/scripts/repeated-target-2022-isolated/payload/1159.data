-- Replacing every input of a+b keeps the updated expression equal to the insert.
INSERT INTO items (a, b) VALUES (2, 1)
ON CONFLICT ((a + b)) DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b;
