-- no-mistakes-disable-file postgres-bounded-statements: intentional synthetic unsupported SQL
INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING;
