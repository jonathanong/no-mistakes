-- no-mistakes-disable-next-line postgres-conflict-ordering: intentional synthetic unsupported SQL
INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING;
