INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING; -- no-mistakes-disable-line postgres-conflict-ordering: intentional synthetic unsupported SQL
