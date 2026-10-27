CREATE TABLE orders (id uuid PRIMARY KEY, created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED);
CREATE TABLE tags (id uuid PRIMARY KEY);
CREATE TABLE invoices (id uuid PRIMARY KEY, created_at timestamptz);
CREATE TABLE public.history (id uuid PRIMARY KEY, created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED);
CREATE TABLE audit.history (id uuid PRIMARY KEY, created_at timestamptz);
ALTER TABLE orders ADD COLUMN alt_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) STORED;
