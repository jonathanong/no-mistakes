INSERT INTO items (id, value) VALUES (1, 2)
ON CONFLICT (id) DO UPDATE SET value = COALESCE(target.value, );
SELECT 42 AS recovered;
