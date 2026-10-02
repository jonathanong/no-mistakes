-- Same-line dynamic DDL must precede the subsequent recreation.
DO $$ BEGIN EXECUTE 'CREATE TABLE orders (id bigint, updated_at timestamptz GENERATED ALWAYS AS (now()) STORED)'; END $$; DROP TABLE orders; CREATE TABLE orders (id bigint, updated_at timestamptz);
-- Direct and dynamic body DDL interleave in source statement order.
DO $$ BEGIN CREATE TABLE other_orders (id bigint, updated_at timestamptz GENERATED ALWAYS AS (now()) STORED); EXECUTE 'DROP TABLE other_orders'; CREATE TABLE other_orders (id bigint, updated_at timestamptz); END $$;
