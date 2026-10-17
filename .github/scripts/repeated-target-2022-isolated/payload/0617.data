/* 雪 */ DO $body$ BEGIN
CREATE VIEW fetched_rows AS SELECT id FROM accounts FETCH FIRST (COALESCE(NULL, 100)) ROWS ONLY;
END $body$;
