-- no-mistakes-disable-file postgres-required-predicates: intentional synthetic unsupported SQL
INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING;
