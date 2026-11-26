INSERT INTO items (id) VALUES (1); -- no-mistakes-disable-line postgres-idempotent-insert: fixture exception
INSERT INTO items (id) VALUES (1);
