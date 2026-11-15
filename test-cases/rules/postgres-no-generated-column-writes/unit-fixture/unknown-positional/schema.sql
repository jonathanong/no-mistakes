CREATE TABLE items (
  id bigint PRIMARY KEY,
  note text,
  created_at timestamptz GENERATED ALWAYS AS (now()) STORED
);
