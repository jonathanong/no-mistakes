-- Each utility must recover as the first action following a validated guard.
CREATE FUNCTION app.utility_0() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
CREATE TYPE "app"."state" AS ENUM (
  'pending', -- Keep quoted enum values and comments in the native AST.
  'done'
);
END;
$$;

CREATE FUNCTION app.utility_1() RETURNS void LANGUAGE plpgsql AS $$
BEGIN IF TRUE THEN
CREATE TRIGGER "audit new" AFTER INSERT ON "app"."events" FOR EACH ROW EXECUTE FUNCTION "app"."audit"('arg', 'other');
END IF; END;
$$;

CREATE FUNCTION app.utility_2() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
CREATE OR REPLACE TRIGGER "audit update" BEFORE UPDATE OF "state" ON "app"."events" FOR EACH ROW WHEN (NEW.state IS DISTINCT FROM OLD.state) EXECUTE FUNCTION "app"."audit"('updated');
END;
$$;

CREATE FUNCTION app.utility_3() RETURNS void LANGUAGE plpgsql AS $$
BEGIN IF TRUE THEN
COMMENT ON FUNCTION "app"."audit"(integer, text[]) IS 'audit metadata';
END IF; END;
$$;

CREATE FUNCTION app.utility_4() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
COMMENT ON TABLE "app"."events" IS 'events';
END;
$$;

CREATE FUNCTION app.utility_5() RETURNS void LANGUAGE plpgsql AS $$
BEGIN IF TRUE THEN
COMMENT ON TRIGGER "audit new" ON "app"."events" IS 'audit hook';
END IF; END;
$$;

CREATE FUNCTION app.utility_6() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
CREATE INDEX "events_state" ON ONLY "app"."events" ("state");
END;
$$;

CREATE FUNCTION app.utility_7() RETURNS void LANGUAGE plpgsql AS $$
BEGIN IF TRUE THEN
ALTER INDEX "app"."events_state" ATTACH PARTITION "app"."events_2026_state";
END IF; END;
$$;

CREATE FUNCTION app.utility_8() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
CREATE TEXT SEARCH CONFIGURATION "app"."search" (COPY = pg_catalog.english);
END;
$$;

CREATE FUNCTION app.utility_9() RETURNS void LANGUAGE plpgsql AS $$
BEGIN IF TRUE THEN
ALTER TEXT SEARCH CONFIGURATION "app"."search" DROP MAPPING FOR asciiword, word;
END IF; END;
$$;

CREATE FUNCTION app.utility_10() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
ALTER TEXT SEARCH CONFIGURATION "app"."search" ADD MAPPING FOR asciiword, word WITH pg_catalog.english_stem, pg_catalog.simple;
END;
$$;

CREATE FUNCTION app.utility_11() RETURNS void LANGUAGE plpgsql AS $$
BEGIN IF TRUE THEN
ALTER TEXT SEARCH CONFIGURATION "app"."search" ALTER MAPPING FOR word WITH pg_catalog.simple;
END IF; END;
$$;

CREATE FUNCTION app.utility_12() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
ALTER TEXT SEARCH CONFIGURATION "app"."search" ALTER MAPPING REPLACE pg_catalog.simple WITH pg_catalog.english_stem;
END;
$$;

CREATE FUNCTION app.utility_13() RETURNS void LANGUAGE plpgsql AS $$
BEGIN IF TRUE THEN
ALTER TEXT SEARCH CONFIGURATION "app"."search" ALTER MAPPING FOR word REPLACE pg_catalog.simple WITH pg_catalog.english_stem;
END IF; END;
$$;

CREATE FUNCTION app.utility_14() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
ALTER TEXT SEARCH CONFIGURATION "app"."search" DROP MAPPING IF EXISTS FOR word, asciiword;
END;
$$;
