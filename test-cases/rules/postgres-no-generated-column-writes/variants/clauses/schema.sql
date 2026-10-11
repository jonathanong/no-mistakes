CREATE TABLE items (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED,
  note text
);
