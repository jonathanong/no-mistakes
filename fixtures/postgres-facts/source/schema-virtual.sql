-- VIRTUAL is valid PostgreSQL 18 syntax; the selected parser still requires STORED.
CREATE TABLE generated_virtual (
    base_value integer,
    computed_value integer GENERATED ALWAYS AS (base_value + 1) VIRTUAL
);
