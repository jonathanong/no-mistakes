-- Ignored plans must not consume an executed INSERT's location or value policy.
EXPLAIN INSERT INTO items (id)                      VALUES (1);
INSERT
INTO items (id) VALUES (2);
EXPLAIN INSERT INTO items (id) VALUES (3);
EXPLAIN ANALYZE
INSERT INTO items (id)                      VALUES (4);
WITH inserted AS (
  INSERT INTO items (id) VALUES (5) RETURNING id
)
INSERT INTO items (id) SELECT id FROM inserted;
WITH ignored_identity AS (
  INSERT INTO items (id)                      VALUES (6) RETURNING id
)
INSERT INTO items (id) SELECT id FROM ignored_identity;
