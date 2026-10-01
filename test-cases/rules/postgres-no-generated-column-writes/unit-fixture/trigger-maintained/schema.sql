CREATE TABLE orders (
  id uuid PRIMARY KEY,
  status text,
  updated_at timestamptz,
  modified_at timestamptz
);

CREATE TABLE invoices (
  id uuid PRIMARY KEY,
  status text
);

CREATE TABLE items (
  id uuid PRIMARY KEY,
  created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED
);
