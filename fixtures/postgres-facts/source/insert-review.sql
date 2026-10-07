-- "conflict" is a boolean column, not an upsert marker.
INSERT INTO accounts SELECT s.id FROM seed s JOIN flags f ON conflict;
INSERT INTO accounts SELECT s.id FROM seed s JOIN flags f ON conflict ON CONFLICT (id) WHERE id > 0 DO NOTHING;
WITH seed AS (SELECT id FROM original) INSERT INTO accounts SELECT id FROM seed ON CONFLICT (id) WHERE id > 0 DO NOTHING;
WITH seed AS (SELECT id FROM original) INSERT INTO accounts SELECT id FROM seed;
-- VALUES and a second WITH are unsupported contexts, never silently flattened.
WITH seed AS (SELECT 1) INSERT INTO accounts VALUES (1);
WITH seed AS (SELECT 1) INSERT INTO accounts DEFAULT VALUES;
WITH seed AS (SELECT 1) INSERT INTO accounts WITH seed AS (SELECT 2) SELECT * FROM seed;
INSERT INTO accounts VALUES (1) ON CONFLICT (id) DO UPDATE SET a = -1, b = +1, c = -1.5, d = -id, e = +EXCLUDED.id, f = NOT active;
DO $$ BEGIN INSERT INTO accounts VALUES (1) RETURNING id; END $$;
DO $$ BEGIN IF active THEN INSERT INTO accounts VALUES (1) RETURNING id; ELSE INSERT INTO accounts VALUES (2); END IF; END $$;
DO $$ BEGIN IF active THEN INSERT INTO accounts VALUES (1); END IF; END $$;
