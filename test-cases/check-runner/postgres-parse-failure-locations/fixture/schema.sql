CREATE TABLE orders (
  id uuid PRIMARY KEY,
  tenant_id uuid,
  created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED
);
