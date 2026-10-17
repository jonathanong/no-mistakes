INSERT INTO items (id, seen) VALUES (1, now())
ON CONFLICT (id) DO UPDATE SET seen = now();
