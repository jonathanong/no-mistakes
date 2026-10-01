CREATE TABLE orders (
  id uuid PRIMARY KEY,
  created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED,
  stored_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED,
  virtual_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) VIRTUAL,
  note_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(note_id)) STORED,
  note_id uuid
);

CREATE TABLE invoices (
  id uuid PRIMARY KEY,
  created_at timestamptz
);

CREATE TABLE tags (
  id uuid PRIMARY KEY
);
