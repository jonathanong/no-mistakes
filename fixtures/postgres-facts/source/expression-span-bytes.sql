DO $$ BEGIN EXECUTE 'SELECT $1' USING now(); END $$;
INSERT INTO t(v) VALUES (coalesce(now(), now()));
INSERT INTO t(v) VALUES ('雪');
INSERT INTO t(v) VALUES (schema.col);
INSERT INTO t(v) SELECT coalesce(now(), 1);
INSERT INTO t(v) VALUES (1), (coalesce(now(), '雪'));
