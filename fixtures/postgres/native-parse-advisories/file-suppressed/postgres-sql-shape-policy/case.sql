-- no-mistakes-disable-file postgres-sql-shape-policy: intentional synthetic unsupported SQL
INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING;
