INSERT INTO events(id) VALUES (1) ON CONFLICT (id) DO NOTHING;
-- A safe sibling must not change this failure's diagnostic.
INSERT INTO events(id) VALUES (;
