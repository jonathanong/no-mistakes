-- These retries must preserve the first table/column definitions and position order.
CREATE TABLE items (id int, generated int GENERATED ALWAYS AS (id + 1) STORED, updated_at int);
CREATE TABLE IF NOT EXISTS items (id int, generated int);
ALTER TABLE items ADD COLUMN IF NOT EXISTS generated int;
ALTER TABLE items ADD COLUMN IF NOT EXISTS updated_at int GENERATED ALWAYS AS (id + 1) STORED;
CREATE TABLE IF NOT EXISTS other (id int);
ALTER TABLE other ADD COLUMN IF NOT EXISTS generated int GENERATED ALWAYS AS (id + 1) STORED;
