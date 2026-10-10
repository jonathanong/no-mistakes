CREATE TABLE items (
  id uuid PRIMARY KEY,
  created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED,
  note text
);

CREATE TABLE logs (
  id uuid PRIMARY KEY,
  note text
);
