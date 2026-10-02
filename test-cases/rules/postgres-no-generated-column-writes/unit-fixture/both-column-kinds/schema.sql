CREATE TABLE orders (id bigint, created_at timestamptz GENERATED ALWAYS AS (now()) STORED, updated_at timestamptz);
