SELECT id INTO TEMP accounts FROM orders LIMIT 1;
SELECT * FROM accounts;
PREPARE metadata_plan AS SELECT id INTO TEMP planned_accounts FROM orders LIMIT 1;
-- Compatibility markers must preserve both the temporary catalog and plans.
COMMENT ON TRIGGER audit ON app.events IS 'hook';
SELECT * FROM accounts;
ALTER INDEX app.events_idx ATTACH PARTITION app.events_2026_idx;
SELECT * FROM accounts;
ALTER TEXT SEARCH CONFIGURATION app.search ADD MAPPING FOR word WITH pg_catalog.simple;
SELECT * FROM accounts;
EXECUTE metadata_plan;
SELECT * FROM planned_accounts;
