CREATE TABLE orders (
 id uuid PRIMARY KEY,
 account_id uuid NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
 details jsonb NOT NULL
);
-- Inline FK syntax in migration SQL agrees with the authoritative saved catalog.
INSERT INTO orders (id, account_id, details)
VALUES ($1, $2, jsonb_build_object('accountId', $2::text, 'total', 10));
