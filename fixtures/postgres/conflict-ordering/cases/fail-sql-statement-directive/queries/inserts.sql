-- deadlock-safe: historical import is externally ordered.
INSERT INTO items (id) SELECT id FROM pending ON CONFLICT (id) DO NOTHING;

-- The nearby directive belongs only to the preceding statement.
INSERT INTO items (id) SELECT id FROM pending ON CONFLICT (id) DO NOTHING;
