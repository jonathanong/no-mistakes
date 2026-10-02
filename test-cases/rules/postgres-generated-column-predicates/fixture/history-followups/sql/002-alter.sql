ALTER TABLE orders ADD COLUMN alt_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED;
