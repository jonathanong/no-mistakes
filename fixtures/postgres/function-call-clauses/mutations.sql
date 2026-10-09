INSERT INTO t (id) VALUES (probe_values()) ON CONFLICT (id) DO UPDATE SET id = probe_set();
INSERT INTO t (id) VALUES (probe_values()) ON CONFLICT DO NOTHING;
INSERT INTO t (id) VALUES (probe_values());
UPDATE t SET id = probe_set() FROM a JOIN b ON probe_from() = b.id;
DELETE FROM t USING a JOIN b ON probe_using() = b.id;
ALTER TABLE t ADD COLUMN id uuid DEFAULT probe_default();
ALTER TABLE t DROP COLUMN id;
MERGE INTO t USING s ON probe_on() = s.id WHEN MATCHED AND probe_when() THEN UPDATE SET id = probe_set() WHEN NOT MATCHED THEN INSERT (id) VALUES (probe_values()) RETURNING probe_returning();
