-- no-mistakes-disable-next-line postgres-required-predicates: intentional synthetic unsupported SQL
INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING;
