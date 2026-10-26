DO $$ BEGIN nonsense CREATE TYPE app.bad AS ENUM ('a'); END $$;
DO $$ BEGIN CREATE TYPE app.bad AS ENUM (; END $$;
DO $$ BEGIN IF THEN COMMENT ON TRIGGER audit ON app.events IS 'x'; END IF; END $$;
DO $$ BEGIN IF TRUE THEN CREATE INDEX bad_idx ON ONLY (state); END IF; END $$;
