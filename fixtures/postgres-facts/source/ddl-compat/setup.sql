-- PostgreSQL 18 execution oracle for valid.sql and after-do.sql.
CREATE SCHEMA app;
CREATE TABLE app.events (id integer, state text) PARTITION BY RANGE (id);
CREATE TABLE app.events_2026 PARTITION OF app.events FOR VALUES FROM (0) TO (100);
CREATE INDEX events_2026_state ON app.events_2026 (state);
CREATE FUNCTION app.audit() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RETURN NEW;
END
$$;
CREATE FUNCTION app.audit(integer, text[]) RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  NULL;
END
$$;
