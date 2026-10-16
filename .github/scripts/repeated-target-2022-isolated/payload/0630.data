INSERT INTO accounts VALUES (1) ON CONFLICT DO NOTHING ON CONFLICT DO NOTHING;
SELECT 42;
-- Recovery must cross the synthetic ON marker and retain one ordinal.
INSERT INTO accounts VALUES (,) ON CONFLICT DO NOTHING;
SELECT 43;
-- A real delimiter resets unmatched parentheses before the next INSERT.
INSERT INTO accounts VALUES (;
INSERT INTO accounts VALUES (1) ON CONFLICT (id) WHERE id > 0 DO NOTHING;
SELECT 44;
