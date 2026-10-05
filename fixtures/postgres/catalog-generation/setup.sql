DROP SCHEMA IF EXISTS "Catalog.Test" CASCADE;
CREATE SCHEMA "Catalog.Test";
CREATE TABLE "Catalog.Test"."Items" (
  "Id" bigint PRIMARY KEY,
  email text NOT NULL,
  active boolean NOT NULL,
  score integer GENERATED ALWAYS AS (length(email)) STORED,
  CONSTRAINT "Email.Unique" UNIQUE (email)
);
CREATE UNIQUE INDEX "partial_desc" ON "Catalog.Test"."Items" (email DESC NULLS LAST, "Id" ASC NULLS FIRST) WHERE active;
CREATE UNIQUE INDEX "expression_key" ON "Catalog.Test"."Items" ((lower(email)) DESC NULLS FIRST);
CREATE INDEX included_key ON "Catalog.Test"."Items" (score) INCLUDE (email);
CREATE TABLE "Catalog.Test".duplicates (value integer);
INSERT INTO "Catalog.Test".duplicates VALUES (1), (1);

CREATE TABLE "Catalog.Test".deferred (value integer CONSTRAINT deferred_key UNIQUE DEFERRABLE INITIALLY DEFERRED);
CREATE TABLE "Catalog.Test".special (value text);
CREATE UNIQUE INDEX special_opclass ON "Catalog.Test".special(value text_pattern_ops);
CREATE UNIQUE INDEX special_collation ON "Catalog.Test".special(value COLLATE "C");

DROP SCHEMA IF EXISTS "Catalog\'Schema" CASCADE;
CREATE SCHEMA "Catalog\'Schema";
CREATE TABLE "Catalog\'Schema".safe (id integer PRIMARY KEY);

CREATE TABLE "Catalog.Test".mixed_collation (value text COLLATE "C");
CREATE UNIQUE INDEX mixed_matching ON "Catalog.Test".mixed_collation(value);
CREATE UNIQUE INDEX mixed_default ON "Catalog.Test".mixed_collation(value COLLATE "default");
CREATE TABLE "Catalog.Test".mixed_deferred (value integer UNIQUE DEFERRABLE INITIALLY DEFERRED);
CREATE UNIQUE INDEX mixed_immediate ON "Catalog.Test".mixed_deferred(value);
CREATE TABLE "Catalog.Test".numeric_expression (value integer);
CREATE UNIQUE INDEX numeric_key ON "Catalog.Test".numeric_expression((value + 1));
-- An expression over a column with a non-default collation derives that collation, so the
-- canonical comparator cannot model it, unlike `lower(email)` above on the default collation.
CREATE UNIQUE INDEX collation_expression ON "Catalog.Test".mixed_collation((lower(value)));
