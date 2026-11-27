INSERT INTO events DEFAULT VALUES;
INSERT INTO events (SELECT '{"a": 1}'::jsonb);
INSERT INTO events SELECT * FROM staged_events;
INSERT INTO events SELECT * FROM staged_events, other;
INSERT INTO events SELECT * FROM (VALUES ('{"k": 1}'::jsonb)) sub;
INSERT INTO events SELECT * FROM staged_events JOIN other ON staged_events.id = other.id;
INSERT INTO events SELECT a.*, '{"b": 2}'::jsonb FROM staged_events a;
