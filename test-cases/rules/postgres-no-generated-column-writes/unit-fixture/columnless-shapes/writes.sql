INSERT INTO votes VALUES (1, now()), (2, now());
INSERT INTO votes SELECT 1, now();
INSERT INTO votes SELECT * FROM source_rows;
WITH seed AS (SELECT 1) INSERT INTO votes VALUES (1, now());
WITH written AS (INSERT INTO votes VALUES (1, now()) RETURNING id) SELECT id FROM written;
INSERT INTO votes VALUES (1, now()) ON CONFLICT (id) DO NOTHING;
MERGE INTO votes USING source_rows ON true WHEN NOT MATCHED THEN INSERT VALUES (1, now());
INSERT INTO votes DEFAULT VALUES;
INSERT INTO items VALUES (1, 'n');
INSERT INTO items VALUES (1, 'n', now());
INSERT INTO items SELECT * FROM source_rows;
INSERT INTO items DEFAULT VALUES;
INSERT INTO votes (id) VALUES (1);
WITH edited AS (UPDATE items SET created_at = now() RETURNING id) SELECT id FROM edited;
WITH noted AS (UPDATE items SET note = 'n' RETURNING id) SELECT id FROM noted;
INSERT INTO votes TABLE source_rows;
INSERT INTO items TABLE source_rows;
WITH removed AS (DELETE FROM items WHERE id = 1 RETURNING id) SELECT id FROM removed;
WITH RECURSIVE seed AS (INSERT INTO items VALUES (1, 'n', now()) RETURNING id), chain AS (SELECT id FROM seed UNION ALL SELECT id FROM chain) SELECT id FROM chain;
WITH outer_rows AS (WITH inner_rows AS (INSERT INTO votes VALUES (1, now()) RETURNING id) SELECT id FROM inner_rows) SELECT id FROM outer_rows;
WITH upserted AS (INSERT INTO items (id, note) VALUES (1, 'n') ON CONFLICT (id) DO UPDATE SET created_at = now() RETURNING id) SELECT id FROM upserted;
WITH placed AS (
INSERT INTO items VALUES (2, 'n', now()) RETURNING id
)
SELECT id FROM placed;
WITH skipped AS (
-- no-mistakes-disable-next-line postgres-no-generated-column-writes
INSERT INTO items VALUES (3, 'n', now()) RETURNING id
)
SELECT id FROM skipped;
