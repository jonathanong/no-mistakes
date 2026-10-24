-- Keep the legacy conjunctive guard when a caller supplies older facts.
INSERT INTO events(id) SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM events WHERE id = 1) COLLATE;
