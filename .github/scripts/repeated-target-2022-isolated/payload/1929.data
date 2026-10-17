-- Empty constant expressions can have no AST span; storage remains source-owned.
CREATE TABLE generated_empty (
    explicit_value integer[] GENERATED ALWAYS AS (ARRAY[]::integer[]) VIRTUAL,
    stored_value integer[] GENERATED ALWAYS AS (ARRAY[]::integer[]) STORED,
    implicit_value integer[] GENERATED ALWAYS AS (ARRAY[]::integer[])
);
ALTER TABLE generated_empty
    ADD COLUMN added_implicit integer[] GENERATED ALWAYS AS (ARRAY[]::integer[]),
    ALTER COLUMN stored_value DROP NOT NULL,
    ADD COLUMN added_stored integer[] GENERATED ALWAYS AS (ARRAY[]::integer[]) STORED,
    ADD COLUMN added_virtual integer[] GENERATED ALWAYS AS (ARRAY[]::integer[]) VIRTUAL;
