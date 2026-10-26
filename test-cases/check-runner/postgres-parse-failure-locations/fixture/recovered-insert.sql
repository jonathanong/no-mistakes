INSERT INTO events(id) VALUES (1) ON CONFLICT (id) WHERE uuidv7() IS NOT NULL DO NOTHING;
-- A rejected later projection cannot publish auxiliary facts from the failed parse.
CREATE RECURSIVE VIEW invalid_view AS SELECT 1;
