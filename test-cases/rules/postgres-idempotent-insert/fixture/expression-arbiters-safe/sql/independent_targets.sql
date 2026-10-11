-- These target expressions are independent, so replacing only a is safe.
INSERT INTO items (a, b) VALUES (2, 'name')
ON CONFLICT (a, lower(b)) DO UPDATE SET a = EXCLUDED.a;
