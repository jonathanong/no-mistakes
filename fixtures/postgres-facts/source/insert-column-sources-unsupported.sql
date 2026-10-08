-- These statements parse; uncertainty belongs to lineage, not syntax errors.
INSERT INTO items VALUES (1, 2);
INSERT INTO items DEFAULT VALUES;
INSERT INTO items (id, value) SELECT * FROM incoming;
INSERT INTO items (id, value) SELECT incoming.* FROM incoming;
INSERT INTO items (id, value) VALUES (1, 2), (3);
INSERT INTO items (id, value) SELECT 1;
INSERT INTO items (id, id) VALUES (1, 2);
INSERT INTO items (id, value) SELECT 1, 2 UNION ALL SELECT 3;
INSERT INTO items (id, value) VALUES (1, ARRAY[2, 3]);
INSERT INTO items (id, value) SELECT 1, 2 UNION BY NAME SELECT 3, 4;
INSERT INTO items (id, value) SELECT 1, 2 UNION ALL (VALUES (3));
