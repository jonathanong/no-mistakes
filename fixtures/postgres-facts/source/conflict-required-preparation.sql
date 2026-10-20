-- An actual policy clause must never silently lose its prepared facts.
INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;
