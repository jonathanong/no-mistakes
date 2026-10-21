CREATE TABLE accounts (id integer, computed integer GENERATED ALWAYS AS (id + 1) STORED);
-- ALTER-only catalogs intentionally lack a complete positional column order.
ALTER TABLE partial_accounts ADD COLUMN computed integer GENERATED ALWAYS AS (id + 1) STORED;
