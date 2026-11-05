-- no-mistakes-disable-next-line postgres-explicit-columns: intentional synthetic unsupported SQL
INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING;
