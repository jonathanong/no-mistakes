CREATE TABLE items (
  id uuid PRIMARY KEY,
  created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED
);
