-- The earlier plain definition must not defeat generated-message precedence.
CREATE TABLE orders (id bigint, updated_at timestamptz);
DROP TABLE orders;
CREATE TABLE orders (id bigint, updated_at timestamptz GENERATED ALWAYS AS (now()) STORED);
