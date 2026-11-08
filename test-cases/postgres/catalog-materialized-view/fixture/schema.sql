DROP SCHEMA IF EXISTS "Catalog.Materialized" CASCADE;
CREATE SCHEMA "Catalog.Materialized";
CREATE TABLE "Catalog.Materialized".events (id bigint PRIMARY KEY, rank integer);
CREATE MATERIALIZED VIEW "Catalog.Materialized".recent_events AS
SELECT id, rank FROM "Catalog.Materialized".events WHERE id >= 100;
CREATE UNIQUE INDEX recent_events_refresh ON "Catalog.Materialized".recent_events (id);
CREATE INDEX recent_events_rank ON "Catalog.Materialized".recent_events (rank DESC);
CREATE VIEW "Catalog.Materialized".ordinary_events AS
SELECT id FROM "Catalog.Materialized".events;
