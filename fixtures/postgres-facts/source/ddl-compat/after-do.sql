-- The following metadata belongs to its own statement, not the preceding body.
DO $$ BEGIN NULL; END $$;
CREATE TYPE "app"."after_do" AS ENUM ('ready');
COMMENT ON FUNCTION "app"."audit"() IS 'after a body';
CREATE INDEX after_do_idx ON ONLY app.events (id);
