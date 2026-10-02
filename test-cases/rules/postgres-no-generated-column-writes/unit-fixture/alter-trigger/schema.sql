CREATE TABLE orders (id bigint, status text, created_at timestamptz GENERATED ALWAYS AS (now()) STORED);
ALTER TABLE orders ADD COLUMN updated_at timestamptz;
ALTER TABLE orders ADD COLUMN IF NOT EXISTS updated_at timestamptz;
-- This migration deliberately omits the external table's CREATE TABLE.
ALTER TABLE external_orders ADD COLUMN updated_at timestamptz;
