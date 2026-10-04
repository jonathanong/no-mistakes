-- PostgreSQL 18 defaults an omitted generated storage mode to VIRTUAL.
CREATE TABLE "Generated" (
    base_value integer,
    implicit_value integer GENERATED ALWAYS AS (base_value + (2 * 3)),
    virtual_value integer GENERATED ALWAYS AS (base_value + 1) VIRTUAL,
    stored_value integer GENERATED ALWAYS AS (base_value + 2) STORED,
    quoted_value text DEFAULT 'GENERATED ALWAYS AS (base_value) VIRTUAL'
);
ALTER TABLE "Generated" ADD COLUMN added_value integer GENERATED ALWAYS AS (base_value + 3);
CREATE TABLE incomplete_generated (value integer GENERATED ALWAYS AS (1;
CREATE TABLE after_generated_error (id integer);
