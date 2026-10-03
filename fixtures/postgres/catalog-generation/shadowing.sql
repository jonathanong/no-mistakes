-- Real-database edge cases that depend on PostgreSQL's own rendering (PostgreSQL 18).
CREATE SCHEMA shadow_demo;
SET search_path = shadow_demo, pg_catalog;

-- An enum named like a pg_catalog type: pg_catalog comes first on the generator's search_path,
-- so a column of this type renders qualified (shadow_demo.text[]), and the enum is keyed alike.
CREATE TYPE text AS ENUM ('draft', 'live');
CREATE TYPE tone AS ENUM ('warm', 'cool');
-- PostgreSQL permits an enum with no labels.
CREATE TYPE empty_kind AS ENUM ();

CREATE TABLE rooms (
  id integer PRIMARY KEY,
  during int4range NOT NULL,
  title pg_catalog.text,
  kinds shadow_demo.text[] NOT NULL DEFAULT '{}',
  tones tone[] NOT NULL DEFAULT '{}',
  CONSTRAINT rooms_no_overlap EXCLUDE USING gist (during WITH &&)
);
CREATE INDEX rooms_during_idx ON rooms USING gist (during);

-- Triggers in every enabled mode: only the ones that fire in a normal session are facts.
CREATE FUNCTION touch_room() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NEW; END $$;
CREATE TRIGGER rooms_live BEFORE UPDATE ON rooms FOR EACH ROW EXECUTE FUNCTION touch_room();
CREATE TRIGGER rooms_off BEFORE UPDATE ON rooms FOR EACH ROW EXECUTE FUNCTION touch_room();
CREATE TRIGGER rooms_replica BEFORE UPDATE ON rooms FOR EACH ROW EXECUTE FUNCTION touch_room();
CREATE TRIGGER rooms_always BEFORE UPDATE ON rooms FOR EACH ROW EXECUTE FUNCTION touch_room();
ALTER TABLE rooms DISABLE TRIGGER rooms_off;
ALTER TABLE rooms ENABLE REPLICA TRIGGER rooms_replica;
ALTER TABLE rooms ENABLE ALWAYS TRIGGER rooms_always;
