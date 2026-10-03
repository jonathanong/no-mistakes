-- A failed concurrent build leaves an observed invalid index in pg_index.
CREATE UNIQUE INDEX CONCURRENTLY invalid_duplicate ON "Catalog.Test".duplicates(value);
