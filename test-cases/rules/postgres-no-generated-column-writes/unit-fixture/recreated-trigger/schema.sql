-- The final ordinary definition must replace the historical generated column.
CREATE TABLE orders (id bigint, old_generated int GENERATED ALWAYS AS (id + 1) STORED, updated_at timestamptz GENERATED ALWAYS AS (now()) STORED);
DROP TABLE orders;
CREATE TABLE orders (id bigint, updated_at timestamptz);
ALTER TABLE orders ADD COLUMN note text;
