-- PostgreSQL rejects a deferrable unique index as an ON CONFLICT arbiter.
INSERT INTO "Catalog.Test".deferred(value) VALUES(1) ON CONFLICT(value) DO NOTHING;
