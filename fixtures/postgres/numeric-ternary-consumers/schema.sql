CREATE TABLE accounts (id uuid PRIMARY KEY, tenant_id uuid, created_at timestamptz);
CREATE TABLE orders (id uuid PRIMARY KEY, details jsonb);
