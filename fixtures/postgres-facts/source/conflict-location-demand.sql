-- Lock/column/star metadata is intentionally irrelevant to conflict-only projection.
SELECT id, count(*) FROM accounts WHERE id > 0 ORDER BY id FOR UPDATE;
INSERT INTO accounts (id) SELECT id FROM accounts ORDER BY id ON CONFLICT (id) DO NOTHING;
