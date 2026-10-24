CREATE TYPE "app"."state" AS ENUM (
  'pending', -- Keep quoted enum values and comments in the native AST.
  'done'
);
CREATE TRIGGER "audit new" AFTER INSERT ON "app"."events" FOR EACH ROW EXECUTE FUNCTION "app"."audit"('arg', 'other');
CREATE OR REPLACE TRIGGER "audit update" BEFORE UPDATE OF "state" ON "app"."events" FOR EACH ROW WHEN (NEW.state IS DISTINCT FROM OLD.state) EXECUTE FUNCTION "app"."audit"('updated');
COMMENT ON FUNCTION "app"."audit"(integer, text[]) IS 'audit metadata';
COMMENT ON TABLE "app"."events" IS 'events';
COMMENT ON TRIGGER "audit new" ON "app"."events" IS 'audit hook';
CREATE INDEX "events_state" ON ONLY "app"."events" ("state");
ALTER INDEX "app"."events_state" ATTACH PARTITION "app"."events_2026_state";
CREATE TEXT SEARCH CONFIGURATION "app"."search" (COPY = pg_catalog.english);
ALTER TEXT SEARCH CONFIGURATION "app"."search" DROP MAPPING FOR asciiword, word;
ALTER TEXT SEARCH CONFIGURATION "app"."search" ADD MAPPING FOR asciiword, word WITH pg_catalog.english_stem, pg_catalog.simple;
ALTER TEXT SEARCH CONFIGURATION "app"."search" ALTER MAPPING FOR word WITH pg_catalog.simple;
ALTER TEXT SEARCH CONFIGURATION "app"."search" ALTER MAPPING REPLACE pg_catalog.simple WITH pg_catalog.english_stem;
ALTER TEXT SEARCH CONFIGURATION "app"."search" ALTER MAPPING FOR word REPLACE pg_catalog.simple WITH pg_catalog.english_stem;
ALTER TEXT SEARCH CONFIGURATION "app"."search" DROP MAPPING IF EXISTS FOR word, asciiword;
