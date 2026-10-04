-- A catalog reader without USAGE cannot search this schema despite visible pg_class rows.
DROP ROLE IF EXISTS no_mistakes_catalog_no_usage;
CREATE ROLE no_mistakes_catalog_no_usage NOLOGIN;
DROP SCHEMA IF EXISTS "Catalog.Locked" CASCADE;
CREATE SCHEMA "Catalog.Locked";
CREATE TABLE "Catalog.Locked".accounts (id uuid);
REVOKE ALL ON SCHEMA "Catalog.Locked" FROM PUBLIC;
